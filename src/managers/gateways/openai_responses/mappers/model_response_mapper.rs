use crate::managers::gateways::openai_responses::resource_models::response_resource::{
    ContentResource, InputTokensDetailsResource, OutputItemResource, OutputTokensDetailsResource, ResponseResource,
    UsageResource,
};
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;

pub struct ModelResponseMapper;

impl ModelResponseMapper {
    /// Expects a resource that has already passed ValidatorResponseResource: each `expect` states a checked fact, and
    /// fails only if the validator is wrong.
    pub fn to_model_response(response_resource: ResponseResource) -> ModelResponse {
        let usage_resource: UsageResource = response_resource.usage.expect("validated by ValidatorResponseResource");
        let output: Vec<OutputItemResource> = response_resource
            .output
            .expect("validated by ValidatorResponseResource");

        ModelResponse {
            id: response_resource.id.expect("validated by ValidatorResponseResource"),
            text: ModelResponseMapper::to_text(output),
            usage: ModelUsage {
                input_tokens: usage_resource
                    .input_tokens
                    .expect("validated by ValidatorResponseResource"),
                // The details are absent when there is nothing to detail.
                cached_input_tokens: usage_resource
                    .input_tokens_details
                    .and_then(|input_tokens_details: InputTokensDetailsResource| input_tokens_details.cached_tokens)
                    .unwrap_or(0),
                output_tokens: usage_resource
                    .output_tokens
                    .expect("validated by ValidatorResponseResource"),
                reasoning_tokens: usage_resource
                    .output_tokens_details
                    .and_then(|output_tokens_details: OutputTokensDetailsResource| {
                        output_tokens_details.reasoning_tokens
                    })
                    .unwrap_or(0),
            },
            service_tier: response_resource
                .service_tier
                .as_deref()
                .and_then(ModelResponseMapper::to_service_tier),
        }
    }

    // The text of every message, in order; reasoning and other items are not part of the answer.
    fn to_text(output: Vec<OutputItemResource>) -> String {
        output
            .into_iter()
            .filter(|output_item: &OutputItemResource| output_item.item_type.as_deref() == Some("message"))
            .flat_map(|output_item: OutputItemResource| output_item.content.unwrap_or_default())
            .filter(|content: &ContentResource| content.content_type.as_deref() == Some("output_text"))
            .filter_map(|content: ContentResource| content.text)
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    fn to_service_tier(service_tier_name: &str) -> Option<ServiceTier> {
        match service_tier_name {
            "default" => Some(ServiceTier::Standard),
            // Some models answer a fast request as served at the priority tier, which is the same service.
            "fast" | "priority" => Some(ServiceTier::Fast),
            "ultrafast" => Some(ServiceTier::Ultrafast),
            _ => None,
        }
    }
}
