use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use ignore::overrides::{Override, OverrideBuilder};
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::workspace_gateway_exception::WorkspaceGatewayException;
use crate::managers::models::search_match::SearchMatch;
use crate::managers::models::workspace_file::WorkspaceFile;

// A file larger than this is not read whole: it is almost never source code, and would fill the model's context.
const MAX_FILE_BYTES: u64 = 512 * 1024;
const MAX_LISTED_FILES: usize = 2_000;
const MAX_SEARCH_MATCHES: usize = 200;
const MAX_MATCH_LINE_CHARS: usize = 300;

/// The workspace: the directory a session's agent works in. It is the only way the agent reads or changes files, so
/// it is where the workspace's boundary is kept: a path is relative to the workspace, never leaves it (not with `..`,
/// not through a symbolic link), and never names a protected file (`.env` files, which hold secrets, and `.git/`).
pub struct WorkspaceGateway;

impl Default for WorkspaceGateway {
    fn default() -> Self {
        WorkspaceGateway::new()
    }
}

impl WorkspaceGateway {
    pub fn new() -> Self {
        WorkspaceGateway
    }

    /// The workspace's absolute, canonical path, after checking it is a directory.
    pub async fn resolve_workspace_directory(&self, directory: &Path) -> ApplicationResult<PathBuf> {
        let context: ContextualData =
            ContextualData::new().with("workspace_directory", directory.to_string_lossy().into_owned());
        let canonical_directory: PathBuf =
            tokio::fs::canonicalize(directory)
                .await
                .map_err(|exception: std::io::Error| {
                    ApplicationBaseException::new(
                        WorkspaceGatewayException::InvalidWorkspace,
                        format!("The workspace {} does not exist", directory.display()),
                        context.clone(),
                    )
                    .with_source(exception)
                })?;
        if !tokio::fs::metadata(&canonical_directory)
            .await
            .map(|metadata: std::fs::Metadata| metadata.is_dir())
            .unwrap_or(false)
        {
            return Err(ApplicationBaseException::new(
                WorkspaceGatewayException::InvalidWorkspace,
                format!("The workspace {} is not a directory", directory.display()),
                context,
            ));
        }
        Ok(canonical_directory)
    }

    /// Makes a new, empty workspace (a new app's folder, and its parents) and returns its canonical path. A folder that
    /// already exists is used only if it is empty, so nothing of someone's is overwritten.
    pub async fn create_workspace_directory(&self, directory: &Path) -> ApplicationResult<PathBuf> {
        let context: ContextualData =
            ContextualData::new().with("workspace_directory", directory.to_string_lossy().into_owned());
        if tokio::fs::try_exists(directory).await.unwrap_or(false) {
            let mut entries: tokio::fs::ReadDir =
                tokio::fs::read_dir(directory)
                    .await
                    .map_err(|exception: std::io::Error| {
                        ApplicationBaseException::new(
                            WorkspaceGatewayException::InvalidWorkspace,
                            format!("{} exists and is not a folder that can be read", directory.display()),
                            context.clone(),
                        )
                        .with_source(exception)
                    })?;
            let has_entry: bool = entries
                .next_entry()
                .await
                .map_err(|exception: std::io::Error| WorkspaceGateway::to_io_exception(exception, &context))?
                .is_some();
            if has_entry {
                return Err(ApplicationBaseException::new(
                    WorkspaceGatewayException::InvalidWorkspace,
                    format!(
                        "{} already holds files; a new app starts in an empty folder",
                        directory.display()
                    ),
                    context,
                ));
            }
        } else {
            tokio::fs::create_dir_all(directory)
                .await
                .map_err(|exception: std::io::Error| WorkspaceGateway::to_io_exception(exception, &context))?;
        }
        self.resolve_workspace_directory(directory).await
    }

    pub async fn read_file(&self, workspace_directory: &Path, path: &str) -> ApplicationResult<WorkspaceFile> {
        let (relative_path, absolute_path): (String, PathBuf) =
            WorkspaceGateway::resolve_path(workspace_directory, path).await?;
        let context: ContextualData = ContextualData::new().with("path", relative_path.as_str());

        let metadata: std::fs::Metadata = tokio::fs::metadata(&absolute_path)
            .await
            .map_err(|exception: std::io::Error| WorkspaceGateway::to_file_exception(exception, &relative_path))?;
        if metadata.is_dir() {
            return Err(ApplicationBaseException::new(
                WorkspaceGatewayException::NotTextFile,
                format!("{relative_path} is a directory; list_files lists what it holds"),
                context,
            ));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(ApplicationBaseException::new(
                WorkspaceGatewayException::NotTextFile,
                format!(
                    "{relative_path} is {} bytes, more than the {MAX_FILE_BYTES} a file may have to be read whole",
                    metadata.len()
                ),
                context,
            ));
        }

        let bytes: Vec<u8> = tokio::fs::read(&absolute_path)
            .await
            .map_err(|exception: std::io::Error| WorkspaceGateway::to_file_exception(exception, &relative_path))?;
        let content: String = String::from_utf8(bytes).map_err(|_: std::string::FromUtf8Error| {
            ApplicationBaseException::new(
                WorkspaceGatewayException::NotTextFile,
                format!("{relative_path} is not a UTF-8 text file"),
                context.clone(),
            )
        })?;
        Ok(WorkspaceFile {
            path: relative_path,
            content,
        })
    }

    pub async fn file_exists(&self, workspace_directory: &Path, path: &str) -> ApplicationResult<bool> {
        let (_, absolute_path): (String, PathBuf) = WorkspaceGateway::resolve_path(workspace_directory, path).await?;
        Ok(tokio::fs::try_exists(&absolute_path).await.unwrap_or(false))
    }

    /// Writes the file, creating its folders. With `create_new`, fails if the file exists.
    pub async fn write_file(
        &self,
        workspace_directory: &Path,
        path: &str,
        content: &str,
        create_new: bool,
    ) -> ApplicationResult<()> {
        let (relative_path, absolute_path): (String, PathBuf) =
            WorkspaceGateway::resolve_path(workspace_directory, path).await?;

        if create_new && tokio::fs::try_exists(&absolute_path).await.unwrap_or(false) {
            return Err(ApplicationBaseException::new(
                WorkspaceGatewayException::FileAlreadyExists,
                format!("{relative_path} already exists; update it instead"),
                ContextualData::new().with("path", relative_path.as_str()),
            ));
        }
        if let Some(parent_directory) = absolute_path.parent() {
            tokio::fs::create_dir_all(parent_directory)
                .await
                .map_err(|exception: std::io::Error| WorkspaceGateway::to_file_exception(exception, &relative_path))?;
        }
        tokio::fs::write(&absolute_path, content)
            .await
            .map_err(|exception: std::io::Error| WorkspaceGateway::to_file_exception(exception, &relative_path))
    }

    pub async fn delete_file(&self, workspace_directory: &Path, path: &str) -> ApplicationResult<()> {
        let (relative_path, absolute_path): (String, PathBuf) =
            WorkspaceGateway::resolve_path(workspace_directory, path).await?;
        tokio::fs::remove_file(&absolute_path)
            .await
            .map_err(|exception: std::io::Error| WorkspaceGateway::to_file_exception(exception, &relative_path))
    }

    /// Every file under `directory` ("" or "." for the whole workspace), as paths relative to the workspace, sorted;
    /// what .gitignore ignores, `.git/` and `.env` files are left out. At most MAX_LISTED_FILES.
    pub async fn list_files(&self, workspace_directory: &Path, directory: &str) -> ApplicationResult<Vec<String>> {
        let (_, absolute_directory): (String, PathBuf) =
            WorkspaceGateway::resolve_path(workspace_directory, directory).await?;
        let root: PathBuf = workspace_directory.to_path_buf();

        tokio::task::spawn_blocking(move || -> Vec<String> {
            let mut paths: Vec<String> = WorkspaceGateway::walk(&absolute_directory, None)
                .filter_map(|entry: DirEntry| WorkspaceGateway::to_relative_path(&root, entry.path()))
                .take(MAX_LISTED_FILES)
                .collect();
            paths.sort();
            paths
        })
        .await
        .map_err(|exception: tokio::task::JoinError| WorkspaceGateway::to_task_exception(exception, "listing files"))
    }

    /// The lines matching `pattern` (a regular expression), in the files `path_glob` matches (all when None). At
    /// most MAX_SEARCH_MATCHES.
    pub async fn search(
        &self,
        workspace_directory: &Path,
        pattern: &str,
        path_glob: Option<&str>,
    ) -> ApplicationResult<Vec<SearchMatch>> {
        let context: ContextualData = ContextualData::new().with("pattern", pattern);
        let regex: Regex = Regex::new(pattern).map_err(|exception: regex::Error| {
            ApplicationBaseException::new(
                WorkspaceGatewayException::InvalidSearchPattern,
                format!("The pattern is not a valid regular expression: {exception}"),
                context.clone(),
            )
        })?;
        let path_override: Option<Override> = match path_glob {
            None => None,
            Some(path_glob) => Some(WorkspaceGateway::to_override(workspace_directory, path_glob)?),
        };
        let root: PathBuf = workspace_directory.to_path_buf();

        tokio::task::spawn_blocking(move || -> Vec<SearchMatch> {
            let mut search_matches: Vec<SearchMatch> = Vec::new();
            for entry in WorkspaceGateway::walk(&root, path_override) {
                let Some(relative_path) = WorkspaceGateway::to_relative_path(&root, entry.path()) else {
                    continue;
                };
                let too_large: bool = entry
                    .metadata()
                    .map(|metadata: std::fs::Metadata| metadata.len() > MAX_FILE_BYTES)
                    .unwrap_or(true);
                if too_large {
                    continue;
                }
                // A file that isn't text holds no lines to match.
                let Ok(content) = std::fs::read_to_string(entry.path()) else {
                    continue;
                };
                for (line_index, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        search_matches.push(SearchMatch {
                            path: relative_path.clone(),
                            line_number: i64::try_from(line_index + 1).unwrap_or(i64::MAX),
                            line: line.chars().take(MAX_MATCH_LINE_CHARS).collect(),
                        });
                        if search_matches.len() >= MAX_SEARCH_MATCHES {
                            return search_matches;
                        }
                    }
                }
            }
            search_matches
        })
        .await
        .map_err(|exception: tokio::task::JoinError| WorkspaceGateway::to_task_exception(exception, "searching"))
    }

    /// Holds nothing open, so there is nothing to close.
    pub fn close(&self) {}

    // The path made relative to the workspace and absolute, after checking it stays inside the workspace and names
    // no protected file. A path that doesn't exist yet is checked through the nearest folder above it that does.
    async fn resolve_path(workspace_directory: &Path, path: &str) -> ApplicationResult<(String, PathBuf)> {
        let context: ContextualData = ContextualData::new().with("path", path);
        let requested_path: &Path = Path::new(path);
        if requested_path.is_absolute() {
            return Err(ApplicationBaseException::new(
                WorkspaceGatewayException::PathOutsideWorkspace,
                format!("{path} is absolute; paths are relative to the workspace"),
                context,
            ));
        }

        let mut relative_path: PathBuf = PathBuf::new();
        for component in requested_path.components() {
            match component {
                Component::Normal(name) => {
                    let name: String = name.to_string_lossy().into_owned();
                    if name.starts_with(".env") || name == ".git" {
                        return Err(ApplicationBaseException::new(
                            WorkspaceGatewayException::ProtectedFile,
                            format!("{path} is protected: .env files and .git/ are never read or changed"),
                            context,
                        ));
                    }
                    relative_path.push(name);
                }
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(ApplicationBaseException::new(
                        WorkspaceGatewayException::PathOutsideWorkspace,
                        format!("{path} leaves the workspace; paths stay inside it"),
                        context,
                    ));
                }
            }
        }

        // A symbolic link can point outside the workspace: what the path really is must be inside it.
        let absolute_path: PathBuf = workspace_directory.join(&relative_path);
        let mut existing_path: &Path = absolute_path.as_path();
        while !tokio::fs::try_exists(existing_path).await.unwrap_or(false) {
            match existing_path.parent() {
                Some(parent_path) => existing_path = parent_path,
                None => break,
            }
        }
        let canonical_path: PathBuf =
            tokio::fs::canonicalize(existing_path)
                .await
                .map_err(|exception: std::io::Error| {
                    WorkspaceGateway::to_file_exception(exception, &relative_path.to_string_lossy())
                })?;
        if !canonical_path.starts_with(workspace_directory) {
            return Err(ApplicationBaseException::new(
                WorkspaceGatewayException::PathOutsideWorkspace,
                format!("{path} leads outside the workspace"),
                context,
            ));
        }

        Ok((relative_path.to_string_lossy().replace('\\', "/"), absolute_path))
    }

    // The files under `directory`, as git would see them: .gitignore applies, `.git/` and `.env` files are skipped.
    fn walk(directory: &Path, path_override: Option<Override>) -> impl Iterator<Item = DirEntry> {
        let mut walk_builder: WalkBuilder = WalkBuilder::new(directory);
        walk_builder
            .hidden(false)
            .require_git(false)
            .filter_entry(|entry: &DirEntry| {
                let name: std::borrow::Cow<'_, str> = entry.file_name().to_string_lossy();
                name != ".git" && !name.starts_with(".env")
            });
        if let Some(path_override) = path_override {
            walk_builder.overrides(path_override);
        }
        walk_builder
            .build()
            // An entry that can't be read is not listed or searched.
            .filter_map(Result::ok)
            .filter(|entry: &DirEntry| {
                entry
                    .file_type()
                    .is_some_and(|file_type: std::fs::FileType| file_type.is_file())
            })
    }

    fn to_override(workspace_directory: &Path, path_glob: &str) -> ApplicationResult<Override> {
        let to_exception = |exception: ignore::Error| -> ApplicationBaseException {
            ApplicationBaseException::new(
                WorkspaceGatewayException::InvalidSearchPattern,
                format!("The path glob is not valid: {exception}"),
                ContextualData::new().with("path_glob", path_glob),
            )
        };
        let mut override_builder: OverrideBuilder = OverrideBuilder::new(workspace_directory);
        override_builder.add(path_glob).map_err(to_exception)?;
        override_builder.build().map_err(to_exception)
    }

    fn to_relative_path(root: &Path, path: &Path) -> Option<String> {
        path.strip_prefix(root)
            .ok()
            .map(|relative_path: &Path| relative_path.to_string_lossy().replace('\\', "/"))
    }

    fn to_file_exception(exception: std::io::Error, relative_path: &str) -> ApplicationBaseException {
        let context: ContextualData = ContextualData::new()
            .with("path", relative_path)
            .with("error_type", format!("{:?}", exception.kind()));
        if exception.kind() == ErrorKind::NotFound {
            return ApplicationBaseException::new(
                WorkspaceGatewayException::FileNotFound,
                format!("{relative_path} does not exist"),
                context,
            );
        }
        ApplicationBaseException::new(
            WorkspaceGatewayException::WorkspaceIo,
            format!("{relative_path} could not be read or written"),
            context,
        )
        .with_source(exception)
    }

    fn to_io_exception(exception: std::io::Error, context: &ContextualData) -> ApplicationBaseException {
        ApplicationBaseException::new(
            WorkspaceGatewayException::WorkspaceIo,
            "The new app's folder could not be made",
            context.clone().with("error_type", format!("{:?}", exception.kind())),
        )
        .with_source(exception)
    }

    fn to_task_exception(exception: tokio::task::JoinError, operation_description: &str) -> ApplicationBaseException {
        ApplicationBaseException::new(
            WorkspaceGatewayException::WorkspaceIo,
            format!("The workspace failed while {operation_description}"),
            ContextualData::new(),
        )
        .with_source(exception)
    }
}
