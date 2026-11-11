use crate::managers::data_layer::managers::gateways::relational_database::resource_models::session_project_maps_row::SessionProjectMapsRow;
use crate::managers::models::session_project_maps::SessionProjectMaps;

pub struct SessionProjectMapsMapper;

impl SessionProjectMapsMapper {
    /// Expects a row that has already passed ValidatorSessionProjectMapsRow; both maps are optional.
    pub fn to_session_project_maps(session_project_maps_row: SessionProjectMapsRow) -> SessionProjectMaps {
        SessionProjectMaps {
            layer_map: session_project_maps_row.layer_map,
            latest_map: session_project_maps_row.latest_map,
        }
    }
}
