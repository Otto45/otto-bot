use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use tokio::io::AsyncWriteExt;

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::data_layer::managers::gateways::session_store::mappers::conversation_item_document_mapper::ConversationItemDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::implementation_brief_document_mapper::ImplementationBriefDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::session_document_mapper::SessionDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::session_request_document_mapper::SessionRequestDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::usage_record_document_mapper::UsageRecordDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::conversation_item_document::ConversationItemDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::project_map_document::ProjectMapDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_document::SessionDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_request_document::SessionRequestDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_conversation_item_document::ValidatorConversationItemDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_session_document::ValidatorSessionDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_session_request_document::ValidatorSessionRequestDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::session::Session;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::session_request::SessionRequest;
use crate::managers::models::usage_record::UsageRecord;

const SESSION_FILE_NAME: &str = "session.json";
// A folder of one conversation per lane, each `<lane_name>.jsonl`.
const CONVERSATION_ITEMS_DIRECTORY_NAME: &str = "conversation_items";
const USAGE_RECORDS_FILE_NAME: &str = "usage_records.jsonl";
const PROJECT_MAPS_FILE_NAME: &str = "project_maps.jsonl";
const BRIEFS_FILE_NAME: &str = "briefs.jsonl";
const REQUESTS_FILE_NAME: &str = "requests.jsonl";

/// Keeps each session in a folder named after its id: `session.json`, and append-only JSON Lines files: the requests
/// it was given, each lane's conversation (every item in the order the model saw it,
/// `conversation_items/<lane_name>.jsonl`), the usage ledger (one line per model response), the project maps and the
/// accepted briefs. Nothing is ever rewritten, so a session's history is never lost to a crash halfway through a
/// write.
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

    /// Keeps a request the session was given, after those before it.
    pub async fn append_session_request(
        &self,
        session_id: &str,
        session_request: &SessionRequest,
    ) -> ApplicationResult<()> {
        let line: String = serde_json::to_string(&SessionRequestDocumentMapper::to_session_request_document(
            session_request,
        ))
        .expect("a session request document serializes to JSON");
        self.append_lines(
            &self.get_file_path(session_id, REQUESTS_FILE_NAME),
            &[line],
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// The requests the session was given, oldest first; none before its first.
    pub async fn get_session_requests(&self, session_id: &str) -> ApplicationResult<Vec<SessionRequest>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let content: String = match tokio::fs::read_to_string(self.get_file_path(session_id, REQUESTS_FILE_NAME)).await
        {
            Ok(content) => content,
            Err(exception) if exception.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(exception) => {
                return Err(SessionStoreGateway::to_io_exception(
                    exception,
                    "reading a session's requests",
                    &context,
                ));
            }
        };

        let mut session_requests: Vec<SessionRequest> = Vec::new();
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number: i64 = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            let mut deserializer: serde_json::Deserializer<serde_json::de::StrRead<'_>> =
                serde_json::Deserializer::from_str(line);
            let document: SessionRequestDocument = serde_path_to_error::deserialize(&mut deserializer).map_err(
                |exception: serde_path_to_error::Error<serde_json::Error>| {
                    ApplicationBaseException::new(
                        SessionStoreGatewayException::InvalidSessionRequestDocument,
                        "A request in the session store could not be deserialized",
                        context
                            .clone()
                            .with("line_number", line_number)
                            .with("errors", ValidationErrorFormatter::format(&exception, "document")),
                    )
                },
            )?;
            ValidatorSessionRequestDocument::validate_session_request_document(&document, session_id, line_number)?;
            session_requests.push(SessionRequestDocumentMapper::to_session_request(document));
        }
        Ok(session_requests)
    }

    /// The lane's conversation so far; none for a lane that hasn't started.
    pub async fn get_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        let context: ContextualData = ContextualData::new()
            .with("session_id", session_id)
            .with("lane_name", lane_name);
        let content: String =
            match tokio::fs::read_to_string(self.get_conversation_items_path(session_id, lane_name)?).await {
                Ok(content) => content,
                Err(exception) if exception.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
                Err(exception) => {
                    return Err(SessionStoreGateway::to_io_exception(
                        exception,
                        "reading a lane's conversation",
                        &context,
                    ));
                }
            };

        let mut conversation_items: Vec<ConversationItem> = Vec::new();
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number: i64 = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            let document: ConversationItemDocument =
                SessionStoreGateway::deserialize_conversation_item_document(line, line_number, &context)?;
            ValidatorConversationItemDocument::validate_conversation_item_document(&document, line_number)?;
            conversation_items.push(ConversationItemDocumentMapper::to_conversation_item(document));
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
        let content: String =
            match tokio::fs::read_to_string(self.get_file_path(session_id, PROJECT_MAPS_FILE_NAME)).await {
                Ok(content) => content,
                Err(exception) if exception.kind() == ErrorKind::NotFound => return Ok(SessionProjectMaps::default()),
                Err(exception) => {
                    return Err(SessionStoreGateway::to_io_exception(
                        exception,
                        "reading a session's project maps",
                        &context,
                    ));
                }
            };

        let mut session_project_maps: SessionProjectMaps = SessionProjectMaps::default();
        for line in content.lines().filter(|line: &&str| !line.trim().is_empty()) {
            let document: ProjectMapDocument = serde_json::from_str(line).map_err(|exception: serde_json::Error| {
                ApplicationBaseException::new(
                    SessionStoreGatewayException::InvalidProjectMapDocument,
                    "A project map in the session store could not be deserialized",
                    context
                        .clone()
                        .with("errors", ValidationErrorFormatter::format_whole(&exception, "document")),
                )
            })?;
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

    // Appends every line in one write, so a run's items land together.
    async fn append_lines(&self, path: &Path, lines: &[String], context: &ContextualData) -> ApplicationResult<()> {
        if lines.is_empty() {
            return Ok(());
        }
        let mut content: String = lines.join("\n");
        content.push('\n');

        let mut file: tokio::fs::File = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(|exception: std::io::Error| {
                SessionStoreGateway::to_io_exception(exception, "opening a session file", context)
            })?;
        file.write_all(content.as_bytes())
            .await
            .map_err(|exception: std::io::Error| {
                SessionStoreGateway::to_io_exception(exception, "appending to a session file", context)
            })?;
        file.flush().await.map_err(|exception: std::io::Error| {
            SessionStoreGateway::to_io_exception(exception, "appending to a session file", context)
        })
    }

    fn deserialize_conversation_item_document(
        line: &str,
        line_number: i64,
        context: &ContextualData,
    ) -> ApplicationResult<ConversationItemDocument> {
        let mut deserializer: serde_json::Deserializer<serde_json::de::StrRead<'_>> =
            serde_json::Deserializer::from_str(line);
        serde_path_to_error::deserialize(&mut deserializer).map_err(
            |exception: serde_path_to_error::Error<serde_json::Error>| {
                ApplicationBaseException::new(
                    SessionStoreGatewayException::InvalidConversationItemDocument,
                    "A conversation item in the session store could not be deserialized",
                    context
                        .clone()
                        .with("line_number", line_number)
                        .with("errors", ValidationErrorFormatter::format(&exception, "document")),
                )
            },
        )
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
