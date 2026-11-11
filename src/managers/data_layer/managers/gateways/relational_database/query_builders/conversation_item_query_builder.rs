use crate::managers::data_layer::managers::gateways::relational_database::resource_models::conversation_item_row::ConversationItemRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};

pub struct ConversationItemQueryBuilder;

impl ConversationItemQueryBuilder {
    // The row an item maps to (ConversationItemMapper) fills the columns its type has; the others are NULL, and the
    // schema checks each type has what it requires.
    pub fn build_append_conversation_item_query(
        session_id: &str,
        lane_name: &str,
        conversation_item_row: &ConversationItemRow,
    ) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "INSERT INTO conversation_items ",
                "(session_id, lane_name, item_type, text, item, call_id, call_kind, output, succeeded, ",
                "reasoning_effort, items) ",
                "VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            ),
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::Text(lane_name.to_string()),
                SqlParameter::OptionalText(conversation_item_row.item_type.clone()),
                SqlParameter::OptionalText(conversation_item_row.text.clone()),
                SqlParameter::OptionalText(conversation_item_row.item.clone()),
                SqlParameter::OptionalText(conversation_item_row.call_id.clone()),
                SqlParameter::OptionalText(conversation_item_row.call_kind.clone()),
                SqlParameter::OptionalText(conversation_item_row.output.clone()),
                SqlParameter::OptionalBoolean(conversation_item_row.succeeded),
                SqlParameter::OptionalText(conversation_item_row.reasoning_effort.clone()),
                SqlParameter::OptionalTextArray(conversation_item_row.items.clone()),
            ],
        }
    }

    /// The lane's conversation as its next request sends it: from its last compaction on (everything before it is in
    /// the compaction), or all of it, in the order the items were appended.
    pub fn build_get_conversation_items_query(session_id: &str, lane_name: &str) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "SELECT item_type, text, item, call_id, call_kind, output, succeeded, reasoning_effort, items ",
                "FROM conversation_items ",
                "WHERE session_id = $1 AND lane_name = $2 AND conversation_item_id >= COALESCE(",
                "(SELECT max(conversation_item_id) FROM conversation_items ",
                "WHERE session_id = $1 AND lane_name = $2 AND item_type = 'compaction'), 0) ",
                "ORDER BY conversation_item_id",
            ),
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::Text(lane_name.to_string()),
            ],
        }
    }
}
