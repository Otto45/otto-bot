use crate::managers::models::service_tier::ServiceTier;
use crate::services::http::wire_models::service_tier_request::ServiceTierRequest;

pub struct ServiceTierRequestMapper;

impl ServiceTierRequestMapper {
    pub fn to_service_tier(service_tier_request: ServiceTierRequest) -> ServiceTier {
        match service_tier_request {
            ServiceTierRequest::Standard => ServiceTier::Standard,
            ServiceTierRequest::Fast => ServiceTier::Fast,
            ServiceTierRequest::Ultrafast => ServiceTier::Ultrafast,
        }
    }
}
