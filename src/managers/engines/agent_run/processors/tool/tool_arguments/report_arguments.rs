use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ReportArguments {
    pub status: Option<String>,
    pub summary: Option<String>,
    pub needs: Option<Vec<String>>,
}
