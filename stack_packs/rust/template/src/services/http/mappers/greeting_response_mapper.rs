use crate::managers::models::greeting::Greeting;
use crate::services::http::wire_models::greeting_response::GreetingResponse;

pub struct GreetingResponseMapper;

impl GreetingResponseMapper {
    pub fn to_greeting_response(greeting: Greeting) -> GreetingResponse {
        GreetingResponse {
            message: greeting.message,
        }
    }
}
