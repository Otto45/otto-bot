/// The project maps a session keeps: the one its requests open with, and the latest a run started with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionProjectMaps {
    pub layer_map: Option<String>,
    pub latest_map: Option<String>,
}
