use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EventQuery {
    /// The id of the last event seen: the events kept after it are sent first (as with a Last-Event-ID header).
    #[serde(default)]
    pub after: Option<u64>,
}
