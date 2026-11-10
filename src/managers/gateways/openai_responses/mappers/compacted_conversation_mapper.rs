use serde_json::Value;

use crate::managers::gateways::openai_responses::mappers::usage_resource_mapper::UsageResourceMapper;
use crate::managers::gateways::openai_responses::resource_models::compaction_resource::CompactionResource;
use crate::managers::models::compacted_conversation::CompactedConversation;

pub struct CompactedConversationMapper;

impl CompactedConversationMapper {
    /// Expects a resource validated by `ValidatorCompactionResource`: its first item, the prompt layers' message, is
    /// left out, and every other is kept as it came.
    pub fn to_compacted_conversation(compaction_resource: CompactionResource, model: &str) -> CompactedConversation {
        CompactedConversation {
            id: compaction_resource
                .id
                .expect("validated by ValidatorCompactionResource"),
            model: model.to_string(),
            items: compaction_resource
                .output
                .expect("validated by ValidatorCompactionResource")
                .iter()
                .skip(1)
                .map(Value::to_string)
                .collect(),
            usage: UsageResourceMapper::to_model_usage(
                compaction_resource
                    .usage
                    .expect("validated by ValidatorCompactionResource"),
            ),
        }
    }
}
