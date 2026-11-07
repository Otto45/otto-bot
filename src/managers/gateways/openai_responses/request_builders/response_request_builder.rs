use serde_json::{Value, json};

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::models::cache_lifetime::CacheLifetime;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::prompt_layer::PromptLayer;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::tool_definition::ToolDefinition;

// Builds the body of a create-response request. The API keeps nothing (`store=false`), so every request carries the
// whole conversation, and the encrypted reasoning the model needs to continue from where it was.
//
// Caching: the developer message opens the request with the prompt layers, each its own content block ending in an
// explicit cache breakpoint with its layer's lifetime. Implicit mode keeps a breakpoint at the end of the latest
// message too, so the conversation is cached as it grows. That is at most three cache writes per request, within the
// API's four.
const APPLY_PATCH: &str = "apply_patch";

pub struct ResponseRequestBuilder;

impl ResponseRequestBuilder {
    /// The whole request: the prompt layers, then every item of the conversation.
    pub fn build(model: &str, model_request: &ModelRequest) -> ApplicationResult<Value> {
        let mut input: Vec<Value> = Vec::with_capacity(model_request.items.len() + 1);
        input.push(json!({
            "role": "developer",
            "content": model_request
                .prompt_layers
                .iter()
                .map(ResponseRequestBuilder::to_cached_content_block)
                .collect::<Vec<Value>>(),
        }));
        for conversation_item in &model_request.items {
            input.push(ResponseRequestBuilder::to_input_item(conversation_item)?);
        }
        Ok(ResponseRequestBuilder::build_body(model, model_request, input, None))
    }

    /// A request that continues `previous_response_id`, which covers the prompt layers and the first
    /// `covered_count` items: only the items after those are sent (WebSocket transport).
    pub fn build_continuation(
        model: &str,
        model_request: &ModelRequest,
        previous_response_id: &str,
        covered_count: usize,
    ) -> ApplicationResult<Value> {
        let input: Vec<Value> = model_request.items[covered_count..]
            .iter()
            .map(ResponseRequestBuilder::to_input_item)
            .collect::<ApplicationResult<Vec<Value>>>()?;
        Ok(ResponseRequestBuilder::build_body(
            model,
            model_request,
            input,
            Some(previous_response_id),
        ))
    }

    fn build_body(
        model: &str,
        model_request: &ModelRequest,
        input: Vec<Value>,
        previous_response_id: Option<&str>,
    ) -> Value {
        let mut tools: Vec<Value> = model_request
            .tools
            .iter()
            .map(ResponseRequestBuilder::to_function_tool)
            .collect();
        tools.push(json!({ "type": APPLY_PATCH }));

        let mut body: Value = json!({
            "model": model,
            "store": false,
            "include": ["reasoning.encrypted_content"],
            "service_tier": ResponseRequestBuilder::to_service_tier_name(model_request.service_tier),
            "parallel_tool_calls": true,
            "prompt_cache_options": { "mode": "implicit" },
            "text": { "verbosity": "low" },
            "tools": tools,
            "input": input,
        });
        if let Some(allowed_tool_names) = model_request.allowed_tool_names.as_ref() {
            // Restricts the calls the model may make without changing the tools sent, so the cached prefix holds.
            body["tool_choice"] = json!({
                "type": "allowed_tools",
                "mode": "auto",
                "tools": allowed_tool_names
                    .iter()
                    .map(|tool_name: &String| ResponseRequestBuilder::to_tool_reference(tool_name))
                    .collect::<Vec<Value>>(),
            });
        }
        if let Some(previous_response_id) = previous_response_id {
            body["previous_response_id"] = Value::from(previous_response_id);
        }
        body
    }

    fn to_input_item(conversation_item: &ConversationItem) -> ApplicationResult<Value> {
        Ok(match conversation_item {
            ConversationItem::UserMessage { text } => json!({
                "role": "user",
                "content": [{ "type": "input_text", "text": text }],
            }),
            ConversationItem::HarnessNote { text } => json!({
                "role": "developer",
                "content": [{ "type": "input_text", "text": text }],
            }),
            ConversationItem::ConfigurationUpdate { reasoning_effort } => json!({
                "type": "configuration_update",
                "reasoning": { "effort": reasoning_effort.name() },
            }),
            // Sent back exactly as the API returned it.
            ConversationItem::ModelOutput { item, .. } => {
                serde_json::from_str::<Value>(item).map_err(|exception: serde_json::Error| {
                    ApplicationBaseException::new(
                        OpenAiResponsesGatewayException::OpenAiRequest,
                        "A model output item of the conversation is not valid JSON",
                        ContextualData::new(),
                    )
                    .with_source(exception)
                })?
            }
            ConversationItem::FunctionCallOutput { call_id, output } => json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": output,
            }),
            ConversationItem::ApplyPatchCallOutput {
                call_id,
                succeeded,
                output,
            } => json!({
                "type": "apply_patch_call_output",
                "call_id": call_id,
                "status": if *succeeded { "completed" } else { "failed" },
                "output": output,
            }),
        })
    }

    fn to_cached_content_block(prompt_layer: &PromptLayer) -> Value {
        json!({
            "type": "input_text",
            "text": prompt_layer.text,
            "prompt_cache_breakpoint": {
                "mode": "explicit",
                "ttl": match prompt_layer.cache_lifetime {
                    CacheLifetime::ThirtyMinutes => "30m",
                    CacheLifetime::TwelveHours => "12h",
                },
            },
        })
    }

    // apply_patch is the API's own tool; every other is a function.
    fn to_tool_reference(tool_name: &str) -> Value {
        if tool_name == APPLY_PATCH {
            json!({ "type": "apply_patch" })
        } else {
            json!({ "type": "function", "name": tool_name })
        }
    }

    // Strict, so the model's arguments always match the tool's schema.
    fn to_function_tool(tool_definition: &ToolDefinition) -> Value {
        json!({
            "type": "function",
            "name": tool_definition.name,
            "description": tool_definition.description,
            "parameters": tool_definition.parameters,
            "strict": true,
        })
    }

    fn to_service_tier_name(service_tier: ServiceTier) -> &'static str {
        match service_tier {
            // The API's name for the standard tier.
            ServiceTier::Standard => "default",
            ServiceTier::Fast => "fast",
            ServiceTier::Ultrafast => "ultrafast",
        }
    }
}
