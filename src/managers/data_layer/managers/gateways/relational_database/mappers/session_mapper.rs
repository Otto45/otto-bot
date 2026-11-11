use std::path::PathBuf;

use crate::managers::data_layer::managers::gateways::relational_database::resource_models::session_row::SessionRow;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session::Session;

pub struct SessionMapper;

impl SessionMapper {
    /// Expects a row that has already passed ValidatorSessionRow: each `expect` states a checked fact, and fails only
    /// if the validator is wrong.
    pub fn to_session(session_row: SessionRow) -> Session {
        let service_tier: String = session_row.service_tier.expect("validated by ValidatorSessionRow");
        Session {
            id: session_row.session_id.expect("validated by ValidatorSessionRow"),
            workspace_directory: PathBuf::from(
                session_row
                    .workspace_directory
                    .expect("validated by ValidatorSessionRow"),
            ),
            service_tier: ServiceTier::from_name(&service_tier).expect("validated by ValidatorSessionRow"),
            started_on: session_row.started_on.expect("validated by ValidatorSessionRow"),
        }
    }
}
