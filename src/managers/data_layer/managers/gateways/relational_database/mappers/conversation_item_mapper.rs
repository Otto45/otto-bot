use crate::managers::data_layer::managers::gateways::relational_database::resource_models::conversation_item_row::ConversationItemRow;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::reasoning_effort::ReasoningEffort;
use crate::managers::models::tool_call_kind::ToolCallKind;
use crate::managers::models::tool_call_reference::ToolCallReference;

pub struct ConversationItemMapper;

impl ConversationItemMapper {
    /// The row an item is inserted as: the columns its type has, the others None.
    pub fn to_conversation_item_row(conversation_item: &ConversationItem) -> ConversationItemRow {
        match conversation_item {
            ConversationItem::UserMessage { text } => ConversationItemRow {
                item_type: Some("user_message".to_string()),
                text: Some(text.clone()),
                ..ConversationItemRow::default()
            },
            ConversationItem::HarnessNote { text } => ConversationItemRow {
                item_type: Some("harness_note".to_string()),
                text: Some(text.clone()),
                ..ConversationItemRow::default()
            },
            ConversationItem::ConfigurationUpdate { reasoning_effort } => ConversationItemRow {
                item_type: Some("configuration_update".to_string()),
                reasoning_effort: Some(reasoning_effort.name().to_string()),
                ..ConversationItemRow::default()
            },
            ConversationItem::ModelOutput { item, tool_call } => ConversationItemRow {
                item_type: Some("model_output".to_string()),
                item: Some(item.clone()),
                call_id: tool_call
                    .as_ref()
                    .map(|tool_call: &ToolCallReference| tool_call.call_id.clone()),
                call_kind: tool_call
                    .as_ref()
                    .map(|tool_call: &ToolCallReference| tool_call.kind.name().to_string()),
                ..ConversationItemRow::default()
            },
            ConversationItem::FunctionCallOutput { call_id, output } => ConversationItemRow {
                item_type: Some("function_call_output".to_string()),
                call_id: Some(call_id.clone()),
                output: Some(output.clone()),
                ..ConversationItemRow::default()
            },
            ConversationItem::ApplyPatchCallOutput {
                call_id,
                succeeded,
                output,
            } => ConversationItemRow {
                item_type: Some("apply_patch_call_output".to_string()),
                call_id: Some(call_id.clone()),
                succeeded: Some(*succeeded),
                output: Some(output.clone()),
                ..ConversationItemRow::default()
            },
            ConversationItem::Compaction { items } => ConversationItemRow {
                item_type: Some("compaction".to_string()),
                items: Some(items.clone()),
                ..ConversationItemRow::default()
            },
        }
    }

    /// Expects a row that has already passed ValidatorConversationItemRow: each `expect` states a checked fact, and
    /// fails only if the validator is wrong.
    pub fn to_conversation_item(conversation_item_row: ConversationItemRow) -> ConversationItem {
        let row: ConversationItemRow = conversation_item_row;
        match row
            .item_type
            .as_deref()
            .expect("validated by ValidatorConversationItemRow")
        {
            "user_message" => ConversationItem::UserMessage {
                text: row.text.expect("validated by ValidatorConversationItemRow"),
            },
            "harness_note" => ConversationItem::HarnessNote {
                text: row.text.expect("validated by ValidatorConversationItemRow"),
            },
            "configuration_update" => ConversationItem::ConfigurationUpdate {
                reasoning_effort: row
                    .reasoning_effort
                    .as_deref()
                    .and_then(ReasoningEffort::from_name)
                    .expect("validated by ValidatorConversationItemRow"),
            },
            "model_output" => ConversationItem::ModelOutput {
                item: row.item.expect("validated by ValidatorConversationItemRow"),
                tool_call: row.call_kind.as_deref().map(|call_kind: &str| ToolCallReference {
                    call_id: row.call_id.clone().expect("validated by ValidatorConversationItemRow"),
                    kind: ToolCallKind::from_name(call_kind).expect("validated by ValidatorConversationItemRow"),
                }),
            },
            "function_call_output" => ConversationItem::FunctionCallOutput {
                call_id: row.call_id.expect("validated by ValidatorConversationItemRow"),
                output: row.output.expect("validated by ValidatorConversationItemRow"),
            },
            "apply_patch_call_output" => ConversationItem::ApplyPatchCallOutput {
                call_id: row.call_id.expect("validated by ValidatorConversationItemRow"),
                succeeded: row.succeeded.expect("validated by ValidatorConversationItemRow"),
                output: row.output.expect("validated by ValidatorConversationItemRow"),
            },
            "compaction" => ConversationItem::Compaction {
                items: row.items.expect("validated by ValidatorConversationItemRow"),
            },
            other => unreachable!("validated by ValidatorConversationItemRow: {other}"),
        }
    }
}
