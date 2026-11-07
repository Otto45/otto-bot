use crate::managers::data_layer::managers::data_manager::DataManager;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::session::Session;
use crate::managers::models::stack_pack::StackPack;
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

    pub async fn get_conversation_items(&self, session_id: &str) -> ApplicationResult<Vec<ConversationItem>> {
        self.data_manager.get_conversation_items(session_id).await
    }

    pub async fn append_conversation_items(
        &self,
        session_id: &str,
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        self.data_manager
            .append_conversation_items(session_id, conversation_items)
            .await
    }

    pub async fn append_usage_record(&self, usage_record: &UsageRecord) -> ApplicationResult<()> {
        self.data_manager.append_usage_record(usage_record).await
    }

    pub async fn get_stack_packs(&self) -> ApplicationResult<Vec<StackPack>> {
        self.data_manager.get_stack_packs().await
    }

    pub async fn get_stack_pack_rules(&self, stack_pack: &StackPack) -> ApplicationResult<String> {
        self.data_manager.get_stack_pack_rules(stack_pack).await
    }

    pub fn close(&self) {
        self.data_manager.close();
    }
}
