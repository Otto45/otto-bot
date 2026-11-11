use std::path::Path;
use std::sync::Arc;

use crate::managers::data_layer::managers::gateways::stack_pack_store::stack_pack_store_gateway::StackPackStoreGateway;
use crate::managers::data_layer::managers::session_storage::SessionStorage;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::session::Session;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::template_file::TemplateFile;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::runtime::held_lock::HeldLock;
use crate::managers::service_locator::service_locator::ServiceLocator;

/// Orchestrates every data gateway: the stack packs' store, and the sessions and their locks wherever configuration
/// keeps them (`SessionStorage`), so that nothing above it learns where a session is kept.
pub struct DataManager {
    session_storage: SessionStorage,
    stack_pack_store_gateway: Arc<StackPackStoreGateway>,
}

impl DataManager {
    pub fn new(service_locator: &ServiceLocator) -> ApplicationResult<Self> {
        let is_relational_database_configured: bool = service_locator
            .get_configuration_provider()?
            .get_storage_settings()
            .relational_database
            .is_some();
        let session_storage: SessionStorage = if is_relational_database_configured {
            SessionStorage::RelationalDatabase {
                relational_database_gateway: service_locator.get_relational_database_gateway()?,
            }
        } else {
            SessionStorage::Files {
                session_store_gateway: service_locator.get_session_store_gateway()?,
                lock_store_gateway: service_locator.get_lock_store_gateway()?,
            }
        };
        Ok(DataManager {
            session_storage,
            stack_pack_store_gateway: service_locator.get_stack_pack_store_gateway()?,
        })
    }

    pub async fn save_session(&self, session: &Session) -> ApplicationResult<()> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.save_session(session).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.save_session(session).await,
        }
    }

    pub async fn get_session(&self, session_id: &str) -> ApplicationResult<Option<Session>> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.get_session(session_id).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.get_session(session_id).await,
        }
    }

    pub async fn get_sessions(&self) -> ApplicationResult<Vec<Session>> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.get_sessions().await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.get_sessions().await,
        }
    }

    pub async fn append_transcript_entry(
        &self,
        session_id: &str,
        transcript_entry: &TranscriptEntry,
    ) -> ApplicationResult<()> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => {
                session_store_gateway
                    .append_transcript_entry(session_id, transcript_entry)
                    .await
            }
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => {
                relational_database_gateway
                    .append_transcript_entry(session_id, transcript_entry)
                    .await
            }
        }
    }

    pub async fn get_transcript(&self, session_id: &str) -> ApplicationResult<Vec<TranscriptEntry>> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.get_transcript(session_id).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.get_transcript(session_id).await,
        }
    }

    pub async fn get_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => {
                session_store_gateway
                    .get_conversation_items(session_id, lane_name)
                    .await
            }
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => {
                relational_database_gateway
                    .get_conversation_items(session_id, lane_name)
                    .await
            }
        }
    }

    pub async fn append_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => {
                session_store_gateway
                    .append_conversation_items(session_id, lane_name, conversation_items)
                    .await
            }
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => {
                relational_database_gateway
                    .append_conversation_items(session_id, lane_name, conversation_items)
                    .await
            }
        }
    }

    pub async fn append_usage_record(&self, usage_record: &UsageRecord) -> ApplicationResult<()> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.append_usage_record(usage_record).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.append_usage_record(usage_record).await,
        }
    }

    pub async fn append_implementation_brief(
        &self,
        session_id: &str,
        implementation_brief: &ImplementationBrief,
    ) -> ApplicationResult<()> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => {
                session_store_gateway
                    .append_implementation_brief(session_id, implementation_brief)
                    .await
            }
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => {
                relational_database_gateway
                    .append_implementation_brief(session_id, implementation_brief)
                    .await
            }
        }
    }

    pub async fn count_implementation_briefs(&self, session_id: &str) -> ApplicationResult<usize> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.count_implementation_briefs(session_id).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => {
                relational_database_gateway
                    .count_implementation_briefs(session_id)
                    .await
            }
        }
    }

    pub async fn get_session_project_maps(&self, session_id: &str) -> ApplicationResult<SessionProjectMaps> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => session_store_gateway.get_session_project_maps(session_id).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.get_session_project_maps(session_id).await,
        }
    }

    pub async fn append_project_map(
        &self,
        session_id: &str,
        project_map: &str,
        project_map_role: ProjectMapRole,
    ) -> ApplicationResult<()> {
        match &self.session_storage {
            SessionStorage::Files {
                session_store_gateway, ..
            } => {
                session_store_gateway
                    .append_project_map(session_id, project_map, project_map_role)
                    .await
            }
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => {
                relational_database_gateway
                    .append_project_map(session_id, project_map, project_map_role)
                    .await
            }
        }
    }

    pub async fn get_stack_packs(&self) -> ApplicationResult<Vec<StackPack>> {
        self.stack_pack_store_gateway.get_stack_packs().await
    }

    pub async fn get_template_files(&self, stack_pack: &StackPack) -> ApplicationResult<Option<Vec<TemplateFile>>> {
        self.stack_pack_store_gateway.get_template_files(stack_pack).await
    }

    pub async fn get_stack_pack_rules(&self, stack_pack: &StackPack) -> ApplicationResult<String> {
        self.stack_pack_store_gateway.get_rules(stack_pack).await
    }

    pub async fn try_lock_app(&self, workspace_directory: &Path) -> ApplicationResult<Option<HeldLock>> {
        match &self.session_storage {
            SessionStorage::Files { lock_store_gateway, .. } => {
                lock_store_gateway.try_lock_app(workspace_directory).await
            }
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.try_lock_app(workspace_directory).await,
        }
    }

    pub async fn lock_session(&self, session_id: &str) -> ApplicationResult<HeldLock> {
        match &self.session_storage {
            SessionStorage::Files { lock_store_gateway, .. } => lock_store_gateway.lock_session(session_id).await,
            SessionStorage::RelationalDatabase {
                relational_database_gateway,
            } => relational_database_gateway.lock_session(session_id).await,
        }
    }

    /// Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
    /// ServiceLocator closes when the Domain Facade closes it.
    pub fn close(&self) {}
}
