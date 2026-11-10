use std::sync::Arc;

use crate::managers::data_layer::managers::gateways::session_store::session_store_gateway::SessionStoreGateway;
use crate::managers::data_layer::managers::gateways::stack_pack_store::stack_pack_store_gateway::StackPackStoreGateway;
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
use crate::managers::service_locator::service_locator::ServiceLocator;

pub struct DataManager {
    session_store_gateway: Arc<SessionStoreGateway>,
    stack_pack_store_gateway: Arc<StackPackStoreGateway>,
}

impl DataManager {
    pub fn new(service_locator: &ServiceLocator) -> ApplicationResult<Self> {
        Ok(DataManager {
            session_store_gateway: service_locator.get_session_store_gateway()?,
            stack_pack_store_gateway: service_locator.get_stack_pack_store_gateway()?,
        })
    }

    pub async fn save_session(&self, session: &Session) -> ApplicationResult<()> {
        self.session_store_gateway.save_session(session).await
    }

    pub async fn get_session(&self, session_id: &str) -> ApplicationResult<Option<Session>> {
        self.session_store_gateway.get_session(session_id).await
    }

    pub async fn get_sessions(&self) -> ApplicationResult<Vec<Session>> {
        self.session_store_gateway.get_sessions().await
    }

    pub async fn append_transcript_entry(
        &self,
        session_id: &str,
        transcript_entry: &TranscriptEntry,
    ) -> ApplicationResult<()> {
        self.session_store_gateway
            .append_transcript_entry(session_id, transcript_entry)
            .await
    }

    pub async fn get_transcript(&self, session_id: &str) -> ApplicationResult<Vec<TranscriptEntry>> {
        self.session_store_gateway.get_transcript(session_id).await
    }

    pub async fn get_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        self.session_store_gateway
            .get_conversation_items(session_id, lane_name)
            .await
    }

    pub async fn append_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        self.session_store_gateway
            .append_conversation_items(session_id, lane_name, conversation_items)
            .await
    }

    pub async fn append_usage_record(&self, usage_record: &UsageRecord) -> ApplicationResult<()> {
        self.session_store_gateway.append_usage_record(usage_record).await
    }

    pub async fn append_implementation_brief(
        &self,
        session_id: &str,
        implementation_brief: &ImplementationBrief,
    ) -> ApplicationResult<()> {
        self.session_store_gateway
            .append_implementation_brief(session_id, implementation_brief)
            .await
    }

    pub async fn count_implementation_briefs(&self, session_id: &str) -> ApplicationResult<usize> {
        self.session_store_gateway.count_implementation_briefs(session_id).await
    }

    pub async fn get_session_project_maps(&self, session_id: &str) -> ApplicationResult<SessionProjectMaps> {
        self.session_store_gateway.get_session_project_maps(session_id).await
    }

    pub async fn append_project_map(
        &self,
        session_id: &str,
        project_map: &str,
        project_map_role: ProjectMapRole,
    ) -> ApplicationResult<()> {
        self.session_store_gateway
            .append_project_map(session_id, project_map, project_map_role)
            .await
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

    /// Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
    /// ServiceLocator closes when the Domain Facade closes it.
    pub fn close(&self) {}
}
