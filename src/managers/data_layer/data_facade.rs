use crate::managers::data_layer::managers::data_manager::DataManager;
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

/// The data layer's only entry point: managers (and the engines they pass it to) never learn where data is kept.
pub struct DataFacade {
    data_manager: DataManager,
}

impl DataFacade {
    pub fn new(service_locator: &ServiceLocator) -> ApplicationResult<Self> {
        Ok(DataFacade {
            data_manager: DataManager::new(service_locator)?,
        })
    }

    pub async fn save_session(&self, session: &Session) -> ApplicationResult<()> {
        self.data_manager.save_session(session).await
    }

    pub async fn get_session(&self, session_id: &str) -> ApplicationResult<Option<Session>> {
        self.data_manager.get_session(session_id).await
    }

    pub async fn get_sessions(&self) -> ApplicationResult<Vec<Session>> {
        self.data_manager.get_sessions().await
    }

    pub async fn append_transcript_entry(
        &self,
        session_id: &str,
        transcript_entry: &TranscriptEntry,
    ) -> ApplicationResult<()> {
        self.data_manager
            .append_transcript_entry(session_id, transcript_entry)
            .await
    }

    pub async fn get_transcript(&self, session_id: &str) -> ApplicationResult<Vec<TranscriptEntry>> {
        self.data_manager.get_transcript(session_id).await
    }

    pub async fn get_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        self.data_manager.get_conversation_items(session_id, lane_name).await
    }

    pub async fn append_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        self.data_manager
            .append_conversation_items(session_id, lane_name, conversation_items)
            .await
    }

    pub async fn append_usage_record(&self, usage_record: &UsageRecord) -> ApplicationResult<()> {
        self.data_manager.append_usage_record(usage_record).await
    }

    pub async fn append_implementation_brief(
        &self,
        session_id: &str,
        implementation_brief: &ImplementationBrief,
    ) -> ApplicationResult<()> {
        self.data_manager
            .append_implementation_brief(session_id, implementation_brief)
            .await
    }

    pub async fn count_implementation_briefs(&self, session_id: &str) -> ApplicationResult<usize> {
        self.data_manager.count_implementation_briefs(session_id).await
    }

    pub async fn get_session_project_maps(&self, session_id: &str) -> ApplicationResult<SessionProjectMaps> {
        self.data_manager.get_session_project_maps(session_id).await
    }

    pub async fn append_project_map(
        &self,
        session_id: &str,
        project_map: &str,
        project_map_role: ProjectMapRole,
    ) -> ApplicationResult<()> {
        self.data_manager
            .append_project_map(session_id, project_map, project_map_role)
            .await
    }

    pub async fn get_stack_packs(&self) -> ApplicationResult<Vec<StackPack>> {
        self.data_manager.get_stack_packs().await
    }

    /// The files a new app of the pack starts with; None if the pack has no template.
    pub async fn get_template_files(&self, stack_pack: &StackPack) -> ApplicationResult<Option<Vec<TemplateFile>>> {
        self.data_manager.get_template_files(stack_pack).await
    }

    pub async fn get_stack_pack_rules(&self, stack_pack: &StackPack) -> ApplicationResult<String> {
        self.data_manager.get_stack_pack_rules(stack_pack).await
    }

    pub fn close(&self) {
        self.data_manager.close();
    }
}
