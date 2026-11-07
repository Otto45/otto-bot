use crate::managers::data_layer::managers::gateways::session_store::resource_models::conversation_item_document::ConversationItemDocument;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::reasoning_effort::ReasoningEffort;
use crate::managers::models::tool_call_kind::ToolCallKind;
use crate::managers::models::tool_call_reference::ToolCallReference;

pub struct ConversationItemDocumentMapper;

impl ConversationItemDocumentMapper {
    pub fn to_conversation_item_document(conversation_item: &ConversationItem) -> ConversationItemDocument {
        match conversation_item {
            ConversationItem::UserMessage { text } => ConversationItemDocument {
                item_type: Some("user_message".to_string()),
                text: Some(text.clone()),
                ..ConversationItemDocument::default()
            },
            ConversationItem::HarnessNote { text } => ConversationItemDocument {
                item_type: Some("harness_note".to_string()),
                text: Some(text.clone()),
                ..ConversationItemDocument::default()
            },
            ConversationItem::ConfigurationUpdate { reasoning_effort } => ConversationItemDocument {
                item_type: Some("configuration_update".to_string()),
                reasoning_effort: Some(reasoning_effort.name().to_string()),
                ..ConversationItemDocument::default()
            },
            ConversationItem::ModelOutput { item, tool_call } => ConversationItemDocument {
                item_type: Some("model_output".to_string()),
                item: Some(item.clone()),
                call_id: tool_call
                    .as_ref()
                    .map(|tool_call: &ToolCallReference| tool_call.call_id.clone()),
                call_kind: tool_call
                    .as_ref()
                    .map(|tool_call: &ToolCallReference| tool_call.kind.name().to_string()),
                ..ConversationItemDocument::default()
            },
            ConversationItem::FunctionCallOutput { call_id, output } => ConversationItemDocument {
                item_type: Some("function_call_output".to_string()),
                call_id: Some(call_id.clone()),
                output: Some(output.clone()),
                ..ConversationItemDocument::default()
            },
            ConversationItem::ApplyPatchCallOutput {
                call_id,
                succeeded,
                output,
            } => ConversationItemDocument {
                item_type: Some("apply_patch_call_output".to_string()),
                call_id: Some(call_id.clone()),
                succeeded: Some(*succeeded),
                output: Some(output.clone()),
                ..ConversationItemDocument::default()
            },
        }
    }

    /// Expects a document that has already passed ValidatorConversationItemDocument: each `expect` states a checked
    /// fact, and fails only if the validator is wrong.
    pub fn to_conversation_item(document: ConversationItemDocument) -> ConversationItem {
        match document
            .item_type
            .as_deref()
            .expect("validated by ValidatorConversationItemDocument")
        {
            "user_message" => ConversationItem::UserMessage {
                text: document.text.expect("validated by ValidatorConversationItemDocument"),
            },
            "harness_note" => ConversationItem::HarnessNote {
                text: document.text.expect("validated by ValidatorConversationItemDocument"),
            },
            "configuration_update" => ConversationItem::ConfigurationUpdate {
                reasoning_effort: document
                    .reasoning_effort
                    .as_deref()
                    .and_then(ReasoningEffort::from_name)
                    .expect("validated by ValidatorConversationItemDocument"),
            },
            "model_output" => ConversationItem::ModelOutput {
                item: document.item.expect("validated by ValidatorConversationItemDocument"),
                tool_call: document.call_kind.as_deref().map(|call_kind: &str| ToolCallReference {
                    call_id: document
                        .call_id
                        .clone()
                        .expect("validated by ValidatorConversationItemDocument"),
                    kind: ToolCallKind::from_name(call_kind).expect("validated by ValidatorConversationItemDocument"),
                }),
            },
            "function_call_output" => ConversationItem::FunctionCallOutput {
                call_id: document
                    .call_id
                    .expect("validated by ValidatorConversationItemDocument"),
                output: document.output.expect("validated by ValidatorConversationItemDocument"),
            },
            "apply_patch_call_output" => ConversationItem::ApplyPatchCallOutput {
                call_id: document
                    .call_id
                    .expect("validated by ValidatorConversationItemDocument"),
                succeeded: document
                    .succeeded
                    .expect("validated by ValidatorConversationItemDocument"),
                output: document.output.expect("validated by ValidatorConversationItemDocument"),
            },
            other => unreachable!("validated by ValidatorConversationItemDocument: {other}"),
        }
    }
}
