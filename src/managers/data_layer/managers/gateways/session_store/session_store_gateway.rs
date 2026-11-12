use std::io::{ErrorKind, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::data_layer::managers::gateways::session_store::mappers::conversation_item_document_mapper::ConversationItemDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::implementation_brief_document_mapper::ImplementationBriefDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::review_baseline_document_mapper::ReviewBaselineDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::run_record_document_mapper::RunRecordDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::session_document_mapper::SessionDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::transcript_entry_document_mapper::TranscriptEntryDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::usage_record_document_mapper::UsageRecordDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::conversation_item_document::ConversationItemDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::project_map_document::ProjectMapDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::review_baseline_document::ReviewBaselineDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::run_record_document::RunRecordDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_document::SessionDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::transcript_entry_document::TranscriptEntryDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_conversation_item_document::ValidatorConversationItemDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_review_baseline_document::ValidatorReviewBaselineDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_run_record_document::ValidatorRunRecordDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_session_document::ValidatorSessionDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_transcript_entry_document::ValidatorTranscriptEntryDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::run_record::RunRecord;
use crate::managers::models::session::Session;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::models::workspace_file::WorkspaceFile;

const SESSION_FILE_NAME: &str = "session.json";
// A folder of one conversation per lane, each `<lane_name>.jsonl`.
const CONVERSATION_ITEMS_DIRECTORY_NAME: &str = "conversation_items";
const USAGE_RECORDS_FILE_NAME: &str = "usage_records.jsonl";
const PROJECT_MAPS_FILE_NAME: &str = "project_maps.jsonl";
const BRIEFS_FILE_NAME: &str = "briefs.jsonl";
const TRANSCRIPT_FILE_NAME: &str = "transcript.jsonl";
const RUN_RECORDS_FILE_NAME: &str = "run_records.jsonl";
const REVIEW_BASELINES_FILE_NAME: &str = "review_baselines.jsonl";

/// Keeps each session in a folder named after its id: `session.json`, and append-only JSON Lines files: its
/// transcript (what the person and the orchestrator said), each lane's conversation (every item in the order the model saw it,
/// `conversation_items/<lane_name>.jsonl`), the usage ledger (one line per model response), the project maps, the
/// accepted briefs, the steps its runs got to and its review baselines. Nothing is ever rewritten, and a write stopped
/// halfway (by a crash, a full disk or a cancelled run) costs the session only the line being written: a line cut short
/// is passed over when the file is read, and the next append starts on a line of its own.
pub struct SessionStoreGateway {
    sessions_directory: PathBuf,
}

impl SessionStoreGateway {
    pub fn new(sessions_directory: PathBuf) -> Self {
        SessionStoreGateway { sessions_directory }
    }

    pub async fn save_session(&self, session: &Session) -> ApplicationResult<()> {
        let session_directory: PathBuf = self.sessions_directory.join(&session.id);
        let context: ContextualData = ContextualData::new().with("session_id", session.id.as_str());

        tokio::fs::create_dir_all(&session_directory)
            .await
            .map_err(|exception: std::io::Error| {
                SessionStoreGateway::to_io_exception(exception, "creating a session's folder", &context)
            })?;

        let mut content: String = serde_json::to_string_pretty(&SessionDocumentMapper::to_session_document(session))
            .expect("a session document serializes to JSON");
        content.push('\n');
        tokio::fs::write(session_directory.join(SESSION_FILE_NAME), content)
            .await
            .map_err(|exception: std::io::Error| {
                SessionStoreGateway::to_io_exception(exception, "saving a session", &context)
            })
    }

    /// The session with the id; None if there is none, or the id couldn't be one (so it never leads out of the
    /// sessions folder).
    pub async fn get_session(&self, session_id: &str) -> ApplicationResult<Option<Session>> {
        if !SessionStoreGateway::is_session_id(session_id) {
            return Ok(None);
        }
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let content: String = match tokio::fs::read_to_string(self.get_file_path(session_id, SESSION_FILE_NAME)).await {
            Ok(content) => content,
            Err(exception) if exception.kind() == ErrorKind::NotFound => return Ok(None),
            Err(exception) => {
                return Err(SessionStoreGateway::to_io_exception(
                    exception,
                    "reading a session",
                    &context,
                ));
            }
        };
        let mut deserializer: serde_json::Deserializer<serde_json::de::StrRead<'_>> =
            serde_json::Deserializer::from_str(&content);
        let document: SessionDocument = serde_path_to_error::deserialize(&mut deserializer).map_err(
            |exception: serde_path_to_error::Error<serde_json::Error>| {
                ApplicationBaseException::new(
                    SessionStoreGatewayException::InvalidSessionDocument,
                    "A session in the session store could not be deserialized",
                    context
                        .clone()
                        .with("errors", ValidationErrorFormatter::format(&exception, "session")),
                )
            },
        )?;
        ValidatorSessionDocument::validate_session_document(&document, session_id)?;
        Ok(Some(SessionDocumentMapper::to_session(document)))
    }

    /// Every session kept, in no order. A folder that isn't a session's is passed over, and so is a session that
    /// can't be read, with a warning: one damaged session doesn't hide the others.
    pub async fn get_sessions(&self) -> ApplicationResult<Vec<Session>> {
        let context: ContextualData = ContextualData::new();
        let mut entries: tokio::fs::ReadDir = match tokio::fs::read_dir(&self.sessions_directory).await {
            Ok(entries) => entries,
            Err(exception) if exception.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(exception) => {
                return Err(SessionStoreGateway::to_io_exception(
                    exception,
                    "listing the sessions",
                    &context,
                ));
            }
        };

        let mut sessions: Vec<Session> = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|exception: std::io::Error| {
            SessionStoreGateway::to_io_exception(exception, "listing the sessions", &context)
        })? {
            let Some(session_id) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            match self.get_session(&session_id).await {
                Ok(Some(session)) => sessions.push(session),
                Ok(None) => {}
                Err(exception) => ApplicationLogger::log_exception(&exception, LogLevel::Warning),
            }
        }
        Ok(sessions)
    }

    /// Keeps something the person or the orchestrator said, after what was said before it.
    pub async fn append_transcript_entry(
        &self,
        session_id: &str,
        transcript_entry: &TranscriptEntry,
    ) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&TranscriptEntryDocumentMapper::to_transcript_entry_document(
            transcript_entry,
        ))
        .expect("a transcript entry document serializes to JSON");
        self.append_lines(
            &self.get_file_path(session_id, TRANSCRIPT_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// What the person and the orchestrator said in the session, oldest first; nothing before its first request.
    pub async fn get_transcript(&self, session_id: &str) -> ApplicationResult<Vec<TranscriptEntry>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let Some(content) = SessionStoreGateway::read_session_file(
            &self.get_file_path(session_id, TRANSCRIPT_FILE_NAME),
            "reading a session's transcript",
            &context,
        )
        .await?
        else {
            return Ok(Vec::new());
        };

        let mut transcript_entries: Vec<TranscriptEntry> = Vec::new();
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number: i64 = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            let Some(document) = SessionStoreGateway::deserialize_line::<TranscriptEntryDocument>(
                line,
                line_number,
                SessionStoreGatewayException::InvalidTranscriptEntryDocument,
                "A transcript entry in the session store could not be deserialized",
                &context,
            )?
            else {
                continue;
            };
            ValidatorTranscriptEntryDocument::validate_transcript_entry_document(&document, session_id, line_number)?;
            transcript_entries.push(TranscriptEntryDocumentMapper::to_transcript_entry(document));
        }
        Ok(transcript_entries)
    }

    /// The lane's conversation as its next request sends it: from its last compaction on (everything before it is in
    /// the compaction), or all of it; none for a lane that hasn't started.
    pub async fn get_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        let context: ContextualData = ContextualData::new()
            .with("session_id", session_id)
            .with("lane_name", lane_name);
        let Some(content) = SessionStoreGateway::read_session_file(
            &self.get_conversation_items_path(session_id, lane_name)?,
            "reading a lane's conversation",
            &context,
        )
        .await?
        else {
            return Ok(Vec::new());
        };

        let mut conversation_items: Vec<ConversationItem> = Vec::new();
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number: i64 = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            let Some(document) = SessionStoreGateway::deserialize_line::<ConversationItemDocument>(
                line,
                line_number,
                SessionStoreGatewayException::InvalidConversationItemDocument,
                "A conversation item in the session store could not be deserialized",
                &context,
            )?
            else {
                continue;
            };
            ValidatorConversationItemDocument::validate_conversation_item_document(&document, line_number)?;
            let conversation_item: ConversationItem = ConversationItemDocumentMapper::to_conversation_item(document);
            if matches!(conversation_item, ConversationItem::Compaction { .. }) {
                conversation_items.clear();
            }
            conversation_items.push(conversation_item);
        }
        Ok(conversation_items)
    }

    pub async fn append_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        let path: PathBuf = self.get_conversation_items_path(session_id, lane_name)?;
        let context: ContextualData = ContextualData::new()
            .with("session_id", session_id)
            .with("lane_name", lane_name);
        if let Some(directory) = path.parent() {
            tokio::fs::create_dir_all(directory)
                .await
                .map_err(|exception: std::io::Error| {
                    SessionStoreGateway::to_io_exception(
                        exception,
                        "creating a session's conversations folder",
                        &context,
                    )
                })?;
        }
        let lines: Vec<String> = conversation_items
            .iter()
            .map(|conversation_item: &ConversationItem| {
                serde_json::to_string(&ConversationItemDocumentMapper::to_conversation_item_document(
                    conversation_item,
                ))
                .expect("a conversation item document serializes to JSON")
            })
            .collect();
        self.append_lines(&path, &lines, &context).await
    }

    pub async fn append_usage_record(&self, usage_record: &UsageRecord) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&UsageRecordDocumentMapper::to_usage_record_document(usage_record))
            .expect("a usage record document serializes to JSON");
        self.append_lines(
            &self.get_file_path(&usage_record.session_id, USAGE_RECORDS_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", usage_record.session_id.as_str()),
        )
        .await
    }

    /// The project maps the session keeps; none before its first run.
    pub async fn get_session_project_maps(&self, session_id: &str) -> ApplicationResult<SessionProjectMaps> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let Some(content) = SessionStoreGateway::read_session_file(
            &self.get_file_path(session_id, PROJECT_MAPS_FILE_NAME),
            "reading a session's project maps",
            &context,
        )
        .await?
        else {
            return Ok(SessionProjectMaps::default());
        };

        let mut session_project_maps: SessionProjectMaps = SessionProjectMaps::default();
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let Some(document) = SessionStoreGateway::deserialize_line::<ProjectMapDocument>(
                line,
                i64::try_from(line_index + 1).unwrap_or(i64::MAX),
                SessionStoreGatewayException::InvalidProjectMapDocument,
                "A project map in the session store could not be deserialized",
                &context,
            )?
            else {
                continue;
            };
            let (Some(project_map), Some(role)) = (document.project_map, document.role) else {
                return Err(ApplicationBaseException::new(
                    SessionStoreGatewayException::InvalidProjectMapDocument,
                    "A project map in the session store is invalid",
                    context.with("errors", "project_map or role is missing"),
                ));
            };
            if role == ProjectMapRole::Layer.name() {
                session_project_maps.layer_map = Some(project_map.clone());
            }
            session_project_maps.latest_map = Some(project_map);
        }
        Ok(session_project_maps)
    }

    pub async fn append_project_map(
        &self,
        session_id: &str,
        project_map: &str,
        project_map_role: ProjectMapRole,
    ) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&ProjectMapDocument {
            project_map: Some(project_map.to_string()),
            role: Some(project_map_role.name().to_string()),
            built_on: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)),
        })
        .expect("a project map document serializes to JSON");
        self.append_lines(
            &self.get_file_path(session_id, PROJECT_MAPS_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// Keeps a brief the harness accepted, after those before it.
    pub async fn append_implementation_brief(
        &self,
        session_id: &str,
        implementation_brief: &ImplementationBrief,
    ) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&ImplementationBriefDocumentMapper::to_implementation_brief_document(
            implementation_brief,
            Utc::now(),
        ))
        .expect("an implementation brief document serializes to JSON");
        self.append_lines(
            &self.get_file_path(session_id, BRIEFS_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// How many briefs the session has accepted, each a line of briefs.jsonl; what they say isn't read.
    pub async fn count_implementation_briefs(&self, session_id: &str) -> ApplicationResult<usize> {
        match tokio::fs::read_to_string(self.get_file_path(session_id, BRIEFS_FILE_NAME)).await {
            Ok(content) => Ok(content.lines().filter(|line: &&str| !line.trim().is_empty()).count()),
            Err(exception) if exception.kind() == ErrorKind::NotFound => Ok(0),
            Err(exception) => Err(SessionStoreGateway::to_io_exception(
                exception,
                "reading a session's briefs",
                &ContextualData::new().with("session_id", session_id),
            )),
        }
    }

    /// Keeps a step one of the session's runs got to, after those before it.
    pub async fn append_run_record(&self, session_id: &str, run_record: &RunRecord) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&RunRecordDocumentMapper::to_run_record_document(run_record))
            .expect("a run record document serializes to JSON");
        self.append_lines(
            &self.get_file_path(session_id, RUN_RECORDS_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// The steps the session's runs got to, oldest first; none before its first run.
    pub async fn get_run_records(&self, session_id: &str) -> ApplicationResult<Vec<RunRecord>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let Some(content) = SessionStoreGateway::read_session_file(
            &self.get_file_path(session_id, RUN_RECORDS_FILE_NAME),
            "reading a session's run records",
            &context,
        )
        .await?
        else {
            return Ok(Vec::new());
        };

        let mut run_records: Vec<RunRecord> = Vec::new();
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number: i64 = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            let Some(document) = SessionStoreGateway::deserialize_line::<RunRecordDocument>(
                line,
                line_number,
                SessionStoreGatewayException::InvalidRunRecordDocument,
                "A run record in the session store could not be deserialized",
                &context,
            )?
            else {
                continue;
            };
            ValidatorRunRecordDocument::validate_run_record_document(&document, session_id, line_number)?;
            run_records.push(RunRecordDocumentMapper::to_run_record(document));
        }
        Ok(run_records)
    }

    /// Keeps what the session's next review is made against: `files`, the workspace's text files as they were when
    /// everything in it had been reviewed; or None, once everything the session built has been reviewed.
    pub async fn append_review_baseline(
        &self,
        session_id: &str,
        files: Option<&[WorkspaceFile]>,
    ) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&ReviewBaselineDocumentMapper::to_review_baseline_document(
            files,
            Utc::now(),
        ))
        .expect("a review baseline document serializes to JSON");
        self.append_lines(
            &self.get_file_path(session_id, REVIEW_BASELINES_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// What the session's next review is made against, as last kept; None when everything it built has been
    /// reviewed, or it has built nothing yet. Only the last line is read as a document: the lines before it are
    /// baselines no longer used.
    pub async fn get_review_baseline(&self, session_id: &str) -> ApplicationResult<Option<Vec<WorkspaceFile>>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let Some(content) = SessionStoreGateway::read_session_file(
            &self.get_file_path(session_id, REVIEW_BASELINES_FILE_NAME),
            "reading a session's review baselines",
            &context,
        )
        .await?
        else {
            return Ok(None);
        };

        let lines: Vec<&str> = content.lines().collect();
        for (line_index, line) in lines.iter().enumerate().rev() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number: i64 = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            let Some(document) = SessionStoreGateway::deserialize_line::<ReviewBaselineDocument>(
                line,
                line_number,
                SessionStoreGatewayException::InvalidReviewBaselineDocument,
                "A review baseline in the session store could not be deserialized",
                &context,
            )?
            else {
                continue;
            };
            ValidatorReviewBaselineDocument::validate_review_baseline_document(&document, session_id, line_number)?;
            return Ok(ReviewBaselineDocumentMapper::to_review_baseline(document));
        }
        Ok(None)
    }

    /// Holds no open file between calls, so there is nothing to close.
    pub fn close(&self) {}

    // A session's id names its folder, so it is a word of lowercase letters and digits, as the ids made are.
    fn is_session_id(session_id: &str) -> bool {
        !session_id.is_empty()
            && session_id
                .chars()
                .all(|character: char| character.is_ascii_lowercase() || character.is_ascii_digit())
    }

    fn get_file_path(&self, session_id: &str, file_name: &str) -> PathBuf {
        self.sessions_directory.join(session_id).join(file_name)
    }

    // The lane's name becomes a file's, so it may hold nothing that leads elsewhere.
    fn get_conversation_items_path(&self, session_id: &str, lane_name: &str) -> ApplicationResult<PathBuf> {
        let is_file_name: bool = !lane_name.is_empty()
            && lane_name.chars().all(|character: char| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            });
        if !is_file_name {
            return Err(ApplicationBaseException::new(
                SessionStoreGatewayException::InvalidLaneName,
                "A lane's name is not snake_case",
                ContextualData::new()
                    .with("session_id", session_id)
                    .with("lane_name", lane_name),
            ));
        }
        Ok(self
            .sessions_directory
            .join(session_id)
            .join(CONVERSATION_ITEMS_DIRECTORY_NAME)
            .join(format!("{lane_name}.jsonl")))
    }

    // Appends every line in one write, so a run's items land together. A file whose last write was cut short ends
    // without a newline: the lines then start on a new one, so the line cut short stays the only one lost.
    async fn append_lines(&self, path: &Path, lines: &[String], context: &ContextualData) -> ApplicationResult<()> {
        if lines.is_empty() {
            return Ok(());
        }
        let to_append_exception = |exception: std::io::Error| -> ApplicationBaseException {
            SessionStoreGateway::to_io_exception(exception, "appending to a session file", context)
        };

        let mut file: tokio::fs::File = tokio::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)
            .await
            .map_err(|exception: std::io::Error| {
                SessionStoreGateway::to_io_exception(exception, "opening a session file", context)
            })?;
        let length: u64 = file.metadata().await.map_err(to_append_exception)?.len();
        let mut content: String = String::new();
        if length > 0 {
            let mut last_byte: [u8; 1] = [0];
            file.seek(SeekFrom::End(-1)).await.map_err(to_append_exception)?;
            file.read_exact(&mut last_byte).await.map_err(to_append_exception)?;
            if last_byte[0] != b'\n' {
                content.push('\n');
            }
        }
        content.push_str(&lines.join("\n"));
        content.push('\n');

        // The file writes at most its buffer's size at a time, with an await between: a run cancelled there would
        // leave half of this. With a buffer as large as the content it is one write, which finishes once started.
        file.set_max_buf_size(content.len());
        file.write_all(content.as_bytes())
            .await
            .map_err(|exception: std::io::Error| {
                SessionStoreGateway::to_io_exception(exception, "appending to a session file", context)
            })?;
        file.flush().await.map_err(|exception: std::io::Error| {
            SessionStoreGateway::to_io_exception(exception, "appending to a session file", context)
        })
    }

    // A session file's text, to read line by line; None for a file that isn't there. Read as bytes: a write cut short
    // can end inside a character, which then belongs to the line that is passed over.
    async fn read_session_file(
        path: &Path,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationResult<Option<String>> {
        match tokio::fs::read(path).await {
            Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
            Err(exception) if exception.kind() == ErrorKind::NotFound => Ok(None),
            Err(exception) => Err(SessionStoreGateway::to_io_exception(
                exception,
                operation_description,
                context,
            )),
        }
    }

    // A line of a session file as its document; None for a line cut short (it ends before its JSON does: a write
    // that a crash, a full disk or a cancelled run stopped halfway), which is passed over with a warning, so it
    // costs the session that line only. Any other line that can't be read is an exception.
    fn deserialize_line<T: DeserializeOwned>(
        line: &str,
        line_number: i64,
        exception_kind: SessionStoreGatewayException,
        message: &str,
        context: &ContextualData,
    ) -> ApplicationResult<Option<T>> {
        let mut deserializer: serde_json::Deserializer<serde_json::de::StrRead<'_>> =
            serde_json::Deserializer::from_str(line);
        match serde_path_to_error::deserialize(&mut deserializer) {
            Ok(document) => Ok(Some(document)),
            Err(exception) if exception.inner().is_eof() => {
                ApplicationLogger::warning(
                    "A line of a session file was cut short; it is passed over",
                    context.clone().with("line_number", line_number),
                    None,
                );
                Ok(None)
            }
            Err(exception) => Err(ApplicationBaseException::new(
                exception_kind,
                message,
                context
                    .clone()
                    .with("line_number", line_number)
                    .with("errors", ValidationErrorFormatter::format(&exception, "document")),
            )),
        }
    }

    fn to_io_exception(
        exception: std::io::Error,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        ApplicationBaseException::new(
            SessionStoreGatewayException::SessionStoreIo,
            format!("The session store failed while {operation_description}"),
            context.clone().with("error_type", format!("{:?}", exception.kind())),
        )
        .with_source(exception)
    }
}
