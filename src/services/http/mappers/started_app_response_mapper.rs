use crate::managers::models::started_project::StartedProject;
use crate::services::http::wire_models::started_app_response::StartedAppResponse;

pub struct StartedAppResponseMapper;

impl StartedAppResponseMapper {
    /// Its folder is left out: apps are addressed by name.
    pub fn to_started_app_response(started_project: StartedProject) -> StartedAppResponse {
        StartedAppResponse {
            name: started_project.app_name.kebab,
            title: started_project.app_name.title,
            stack: started_project.stack_name,
            file_count: started_project.file_count,
            formatted: started_project.formatted,
        }
    }
}
