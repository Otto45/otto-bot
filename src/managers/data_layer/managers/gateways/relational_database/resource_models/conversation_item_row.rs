// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
// Which columns an item has depends on its type.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct ConversationItemRow {
    pub item_type: Option<String>,
    pub text: Option<String>,
    pub item: Option<String>,
    pub call_id: Option<String>,
    // For a model output that is a tool call: which kind (its call_id is in call_id).
    pub call_kind: Option<String>,
    pub output: Option<String>,
    pub succeeded: Option<bool>,
    pub reasoning_effort: Option<String>,
    // A compaction's items, each as the API returned it.
    pub items: Option<Vec<String>>,
}
