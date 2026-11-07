use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use tokio::io::AsyncWriteExt;

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::data_layer::managers::gateways::session_store::mappers::conversation_item_document_mapper::ConversationItemDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::session_document_mapper::SessionDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::mappers::usage_record_document_mapper::UsageRecordDocumentMapper;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::conversation_item_document::ConversationItemDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::project_map_document::ProjectMapDocument;
use crate::managers::data_layer::managers::gateways::session_store::validators::validator_conversation_item_document::ValidatorConversationItemDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::session::Session;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::usage_record::UsageRecord;

const SESSION_FILE_NAME: &str = "session.json";
const CONVERSATION_ITEMS_FILE_NAME: &str = "conversation_items.jsonl";
const USAGE_RECORDS_FILE_NAME: &str = "usage_records.jsonl";
const PROJECT_MAPS_FILE_NAME: &str = "project_maps.jsonl";

/// Keeps each session in a folder of its own: `session.json`, and two append-only JSON Lines files, its conversation
/// (every item in the order the model saw it) and its usage ledger (one line per model response). Nothing is ever
/// rewritten, so a session's history is never lost to a crash halfway through a write.
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

    /// The session's conversation so far; none for a session that has had no request yet.
    pub async fn get_conversation_items(&self, session_id: &str) -> ApplicationResult<Vec<ConversationItem>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let content: String =
            match tokio::fs::read_to_string(self.get_file_path(session_id, CONVERSATION_ITEMS_FILE_NAME)).await {
                Ok(content) => content,
                Err(exception) if exception.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
                Err(exception) => {
                    return Err(SessionStoreGateway::to_io_exception(
                        exception,
                        "reading a session's conversation",
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
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        let lines: Vec<String> = conversation_items
            .iter()
            .map(|conversation_item: &ConversationItem| {
                serde_json::to_string(&ConversationItemDocumentMapper::to_conversation_item_document(
                    conversation_item,
                ))
                .expect("a conversation item document serializes to JSON")
            })
            .collect();
        self.append_lines(
            &self.get_file_path(session_id, CONVERSATION_ITEMS_FILE_NAME),
            &lines,
            &ContextualData::new().with("session_id", session_id),
        )
        .await
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

    /// Holds no open file between calls, so there is nothing to close.
    pub fn close(&self) {}

    fn get_file_path(&self, session_id: &str, file_name: &str) -> PathBuf {
        self.sessions_directory.join(session_id).join(file_name)
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
