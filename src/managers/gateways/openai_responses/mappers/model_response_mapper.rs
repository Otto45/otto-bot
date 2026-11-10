use serde_json::Value;

use crate::managers::gateways::openai_responses::mappers::usage_resource_mapper::UsageResourceMapper;
use crate::managers::gateways::openai_responses::resource_models::output_item_resource::{
    ContentResource, OutputItemResource, PatchOperationResource,
};
use crate::managers::gateways::openai_responses::resource_models::response_resource::{
    ResponseResource, UsageResource,
};
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::patch_operation::PatchOperation;
use crate::managers::models::patch_operation_kind::PatchOperationKind;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::tool_call::ToolCall;
use crate::managers::models::tool_call_kind::ToolCallKind;
use crate::managers::models::tool_call_reference::ToolCallReference;

pub struct ModelResponseMapper;

impl ModelResponseMapper {
    /// Expects a resource that has already passed ValidatorResponseResource: each `expect` states a checked fact, and
    /// fails only if the validator is wrong.
    pub fn to_model_response(response_resource: ResponseResource) -> ModelResponse {
        let usage_resource: UsageResource = response_resource.usage.expect("validated by ValidatorResponseResource");
        let output: Vec<Value> = response_resource
            .output
            .expect("validated by ValidatorResponseResource");
        let output_item_resources: Vec<OutputItemResource> = output
            .iter()
            .map(|output_item: &Value| {
                serde_json::from_value::<OutputItemResource>(output_item.clone())
                    .expect("validated by ValidatorResponseResource")
            })
            .collect();

        let texts: Vec<String> = output_item_resources
            .iter()
            .filter(|output_item_resource: &&OutputItemResource| {
                output_item_resource.item_type.as_deref() == Some("message")
            })
            .flat_map(|output_item_resource: &OutputItemResource| output_item_resource.content.iter().flatten())
            .filter(|content: &&ContentResource| content.content_type.as_deref() == Some("output_text"))
            .filter_map(|content: &ContentResource| content.text.clone())
            .collect();

        ModelResponse {
            id: response_resource.id.expect("validated by ValidatorResponseResource"),
            model: response_resource.model.expect("validated by ValidatorResponseResource"),
            output_items: output
                .iter()
                .zip(output_item_resources.iter())
                .map(|(output_item, output_item_resource): (&Value, &OutputItemResource)| {
                    ConversationItem::ModelOutput {
                        item: output_item.to_string(),
                        tool_call: ModelResponseMapper::to_tool_call_reference(output_item_resource),
                    }
                })
                .collect(),
            text: (!texts.is_empty()).then(|| texts.join("\n\n")),
            tool_calls: output_item_resources
                .into_iter()
                .filter_map(ModelResponseMapper::to_tool_call)
                .collect(),
            usage: UsageResourceMapper::to_model_usage(usage_resource),
            service_tier: response_resource
                .service_tier
                .as_deref()
                .and_then(ModelResponseMapper::to_service_tier),
        }
    }

    fn to_tool_call_reference(output_item_resource: &OutputItemResource) -> Option<ToolCallReference> {
        let kind: ToolCallKind = match output_item_resource.item_type.as_deref() {
            Some("function_call") => ToolCallKind::Function,
            Some("apply_patch_call") => ToolCallKind::ApplyPatch,
            _ => return None,
        };
        Some(ToolCallReference {
            call_id: output_item_resource
                .call_id
                .clone()
                .expect("validated by ValidatorResponseResource"),
            kind,
        })
    }

    fn to_tool_call(output_item_resource: OutputItemResource) -> Option<ToolCall> {
        match output_item_resource.item_type.as_deref() {
            Some("function_call") => Some(ToolCall::Function {
                call_id: output_item_resource
                    .call_id
                    .expect("validated by ValidatorResponseResource"),
                name: output_item_resource
                    .name
                    .expect("validated by ValidatorResponseResource"),
                arguments: output_item_resource
                    .arguments
                    .expect("validated by ValidatorResponseResource"),
            }),
            Some("apply_patch_call") => {
                let operation: PatchOperationResource = output_item_resource
                    .operation
                    .expect("validated by ValidatorResponseResource");
                Some(ToolCall::ApplyPatch {
                    call_id: output_item_resource
                        .call_id
                        .expect("validated by ValidatorResponseResource"),
                    patch_operation: PatchOperation {
                        kind: match operation.operation_type.as_deref() {
                            Some("create_file") => PatchOperationKind::CreateFile,
                            Some("update_file") => PatchOperationKind::UpdateFile,
                            Some("delete_file") => PatchOperationKind::DeleteFile,
                            other => unreachable!("validated by ValidatorResponseResource: {other:?}"),
                        },
                        path: operation.path.expect("validated by ValidatorResponseResource"),
                        diff: operation.diff,
                    },
                })
            }
            _ => None,
        }
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
