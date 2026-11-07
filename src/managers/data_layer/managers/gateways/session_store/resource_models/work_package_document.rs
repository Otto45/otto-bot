use serde::{Deserialize, Serialize};

// A package of a work plan, as plans.jsonl keeps it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkPackageDocument {
    pub name: Option<String>,
    pub brief: Option<String>,
    pub owned_paths: Option<Vec<String>>,
}
