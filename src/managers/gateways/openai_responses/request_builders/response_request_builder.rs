use serde_json::{Value, json};

use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::service_tier::ServiceTier;

// Builds the body of a create-response request.
pub struct ResponseRequestBuilder;

impl ResponseRequestBuilder {
    pub fn build(model: &str, model_request: &ModelRequest) -> Value {
        json!({
            "model": model,
            // The harness keeps every conversation itself; nothing is stored by the API.
            "store": false,
            "service_tier": ResponseRequestBuilder::to_service_tier_name(model_request.service_tier),
            "input": [
                {
                    "role": "user",
                    "content": [{ "type": "input_text", "text": model_request.user_text }],
                },
            ],
        })
    }

    fn to_service_tier_name(service_tier: ServiceTier) -> &'static str {
        match service_tier {
            // The API's name for the standard tier.
            ServiceTier::Standard => "default",
            ServiceTier::Fast => "fast",
            ServiceTier::Ultrafast => "ultrafast",
        }
    }
}
