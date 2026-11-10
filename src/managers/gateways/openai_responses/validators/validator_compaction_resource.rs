use serde_json::Value;

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::resource_models::compaction_resource::CompactionResource;
use crate::managers::gateways::openai_responses::validators::validator_response_resource::ValidatorResponseResource;

const COMPACTION_ITEM_TYPE: &str = "compaction";

pub struct ValidatorCompactionResource;

impl ValidatorCompactionResource {
    /// A compacted conversation opens with the prompt layers' message it was sent, unchanged (requests open with it
    /// anyway, so it is left out of what the lane keeps), and holds a compaction item.
    pub fn validate_compaction_resource(
        compaction_resource: &CompactionResource,
        prompt_layers_message: &Value,
    ) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        if compaction_resource.id.as_deref().is_none_or(str::is_empty) {
            errors.push("id is missing".to_string());
        }
        match compaction_resource.usage.as_ref() {
            None => errors.push("usage is missing".to_string()),
            Some(usage_resource) => errors.extend(ValidatorResponseResource::validate_usage_resource(usage_resource)),
        }
        match compaction_resource.output.as_deref() {
            None | Some([]) => errors.push("output is missing".to_string()),
            Some([first_item, later_items @ ..]) => {
                let opens_with_prompt_layers: bool = first_item.get("role").and_then(Value::as_str)
                    == Some("developer")
                    && first_item.get("content") == prompt_layers_message.get("content");
                if !opens_with_prompt_layers {
                    errors.push("output[0] is not the prompt layers' message it was sent".to_string());
                }
                let has_compaction_item: bool = later_items
                    .iter()
                    .any(|item: &Value| item.get("type").and_then(Value::as_str) == Some(COMPACTION_ITEM_TYPE));
                if !has_compaction_item {
                    errors.push("output has no compaction item".to_string());
                }
            }
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                OpenAiResponsesGatewayException::InvalidOpenAiResponse,
                "The compacted conversation is invalid",
                ContextualData::new()
                    .with("resource_name", "compaction")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
