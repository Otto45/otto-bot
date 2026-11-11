use crate::managers::data_layer::managers::gateways::relational_database::resource_models::conversation_item_row::ConversationItemRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::managers::models::reasoning_effort::ReasoningEffort;
use crate::managers::models::tool_call_kind::ToolCallKind;

pub struct ValidatorConversationItemRow;

impl ValidatorConversationItemRow {
    /// Which columns an item needs depends on its type.
    pub fn validate_conversation_item_row(
        conversation_item_row: &ConversationItemRow,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<()> {
        let row: &ConversationItemRow = conversation_item_row;
        let checks: Vec<(&str, bool)> = match row.item_type.as_deref() {
            Some("user_message") | Some("harness_note") => vec![("text", row.text.is_some())],
            Some("configuration_update") => vec![(
                "reasoning_effort (one of low, medium, high, xhigh, max)",
                row.reasoning_effort
                    .as_deref()
                    .and_then(ReasoningEffort::from_name)
                    .is_some(),
            )],
            Some("model_output") => vec![
                ("item", row.item.is_some()),
                (
                    "call_kind (function or apply_patch)",
                    row.call_kind
                        .as_deref()
                        .is_none_or(|call_kind: &str| ToolCallKind::from_name(call_kind).is_some()),
                ),
                // A tool call is answered by its call_id.
                ("call_id", row.call_kind.is_none() || row.call_id.is_some()),
            ],
            Some("function_call_output") => vec![("call_id", row.call_id.is_some()), ("output", row.output.is_some())],
            Some("apply_patch_call_output") => vec![
                ("call_id", row.call_id.is_some()),
                ("succeeded", row.succeeded.is_some()),
                ("output", row.output.is_some()),
            ],
            Some("compaction") => vec![(
                "items (at least one)",
                row.items.as_ref().is_some_and(|items: &Vec<String>| !items.is_empty()),
            )],
            Some(_) | None => vec![("item_type (a conversation item type)", false)],
        };
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresConversationItemRow,
                "Conversation item row from the Postgres conversation_items table failed validation",
                ContextualData::new()
                    .with("resource_name", "conversation_item")
                    .with("session_id", session_id)
                    .with("lane_name", lane_name)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
