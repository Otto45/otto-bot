use std::sync::Arc;

use tokio::sync::broadcast;

use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::gateways::openai_responses::openai_responses_gateway::OpenAiResponsesGateway;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::user_request::UserRequest;

// Runs one agent on one request, to completion: the only component that talks to the model. It reports what happens
// as it happens, on the session's events. Today a run is one model response; the tool loop (model turn -> tool calls
// -> results -> ...) grows here.
pub struct AgentRunEngine {
    openai_responses_gateway: Arc<OpenAiResponsesGateway>,
}

impl AgentRunEngine {
    pub fn new(openai_responses_gateway: Arc<OpenAiResponsesGateway>) -> Self {
        AgentRunEngine {
            openai_responses_gateway,
        }
    }

    /// Reports what happens on `events`; the run's start and end are the manager's to report.
    pub async fn run(
        &self,
        session: &Session,
        user_request: &UserRequest,
        events: &broadcast::Sender<SessionEvent>,
    ) -> ApplicationResult<()> {
        let model_request: ModelRequest = ModelRequest {
            user_text: user_request.text.clone(),
            service_tier: session.service_tier,
        };
        let model_response: ModelResponse = self.openai_responses_gateway.create_response(&model_request).await?;

        // A session may have no client watching it; what isn't received is not an error.
        events
            .send(SessionEvent::ModelResponded {
                text: model_response.text,
                usage: model_response.usage,
                service_tier: model_response.service_tier,
            })
            .ok();
        Ok(())
    }
}
