use crate::managers::gateways::openai_responses::resource_models::response_resource::{
    InputTokensDetailsResource, OutputTokensDetailsResource, UsageResource,
};
use crate::managers::models::model_usage::ModelUsage;

pub struct UsageResourceMapper;

impl UsageResourceMapper {
    /// Expects a usage that has passed `ValidatorResponseResource::validate_usage_resource`.
    pub fn to_model_usage(usage_resource: UsageResource) -> ModelUsage {
        ModelUsage {
            input_tokens: usage_resource
                .input_tokens
                .expect("validated by ValidatorResponseResource"),
            // The details are absent when there is nothing to detail.
            cached_input_tokens: usage_resource
                .input_tokens_details
                .as_ref()
                .and_then(|input_tokens_details: &InputTokensDetailsResource| input_tokens_details.cached_tokens)
                .unwrap_or(0),
            cache_write_tokens: usage_resource
                .input_tokens_details
                .as_ref()
                .and_then(|input_tokens_details: &InputTokensDetailsResource| input_tokens_details.cache_write_tokens)
                .unwrap_or(0),
            output_tokens: usage_resource
                .output_tokens
                .expect("validated by ValidatorResponseResource"),
            reasoning_tokens: usage_resource
                .output_tokens_details
                .and_then(|output_tokens_details: OutputTokensDetailsResource| output_tokens_details.reasoning_tokens)
                .unwrap_or(0),
        }
    }
}
