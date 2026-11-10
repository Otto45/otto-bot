use crate::managers::data_layer::managers::gateways::session_store::resource_models::conversation_item_document::ConversationItemDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::models::reasoning_effort::ReasoningEffort;
use crate::managers::models::tool_call_kind::ToolCallKind;

pub struct ValidatorConversationItemDocument;

impl ValidatorConversationItemDocument {
    pub fn validate_conversation_item_document(
        document: &ConversationItemDocument,
        line_number: i64,
    ) -> ApplicationResult<()> {
        let required: Vec<(&str, bool)> = match document.item_type.as_deref() {
            Some("user_message") | Some("harness_note") => vec![("text", document.text.is_some())],
            Some("configuration_update") => vec![(
                "reasoning_effort (one of low, medium, high, xhigh, max)",
                document
                    .reasoning_effort
                    .as_deref()
                    .and_then(ReasoningEffort::from_name)
                    .is_some(),
            )],
            Some("model_output") => {
                if let Some(call_kind) = document.call_kind.as_deref()
                    && ToolCallKind::from_name(call_kind).is_none()
                {
                    return Err(ValidatorConversationItemDocument::to_exception(
                        "call_kind is not a tool call kind".to_string(),
                        line_number,
                    ));
                }
                vec![
                    ("item", document.item.is_some()),
                    // A tool call is answered by its call_id.
                    ("call_id", document.call_kind.is_none() || document.call_id.is_some()),
                ]
            }
            Some("function_call_output") => vec![
                ("call_id", document.call_id.is_some()),
                ("output", document.output.is_some()),
            ],
            Some("apply_patch_call_output") => vec![
                ("call_id", document.call_id.is_some()),
                ("succeeded", document.succeeded.is_some()),
                ("output", document.output.is_some()),
            ],
            Some("compaction") => vec![(
                "items (at least one)",
                document
                    .items
                    .as_ref()
                    .is_some_and(|items: &Vec<String>| !items.is_empty()),
            )],
            Some(_) => {
                return Err(ValidatorConversationItemDocument::to_exception(
                    "type is not a conversation item type".to_string(),
                    line_number,
                ));
            }
            None => {
                return Err(ValidatorConversationItemDocument::to_exception(
                    "type is missing".to_string(),
                    line_number,
                ));
            }
        };

        let errors: Vec<String> = required
            .into_iter()
            .filter(|(_, present): &(&str, bool)| !present)
            .map(|(property_name, _): (&str, bool)| format!("{property_name} is missing"))
            .collect();
        if !errors.is_empty() {
            return Err(ValidatorConversationItemDocument::to_exception(
                errors.join("; "),
                line_number,
            ));
        }
        Ok(())
    }

    fn to_exception(errors: String, line_number: i64) -> ApplicationBaseException {
        ApplicationBaseException::new(
            SessionStoreGatewayException::InvalidConversationItemDocument,
            "A conversation item in the session store is invalid",
            ContextualData::new()
                .with("resource_name", "conversation_item")
                .with("line_number", line_number)
                .with("errors", errors),
        )
    }
}
