use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::resource_models::response_resource::{
    ContentResource, IncompleteDetailsResource, OutputItemResource, ResponseResource, UsageResource,
};

pub struct ValidatorResponseResource;

impl ValidatorResponseResource {
    pub fn validate_response_resource(response_resource: &ResponseResource) -> ApplicationResult<()> {
        // A response that stopped early is not malformed, but there is no complete answer to use.
        if let Some(status) = response_resource.status.as_deref()
            && status != "completed"
        {
            let mut context: ContextualData = ContextualData::new().with("status", status);
            if let Some(reason) = response_resource
                .incomplete_details
                .as_ref()
                .and_then(|incomplete_details: &IncompleteDetailsResource| incomplete_details.reason.as_deref())
            {
                context.insert("incomplete_reason", reason);
            }
            return Err(ApplicationBaseException::new(
                OpenAiResponsesGatewayException::IncompleteOpenAiResponse,
                "The model's response did not complete",
                context,
            ));
        }

        let mut errors: Vec<String> = Vec::new();
        for (property_name, property_value) in [
            ("id", response_resource.id.as_deref()),
            ("status", response_resource.status.as_deref()),
        ] {
            if let Some(error) = ValidatorString::validate(property_name, property_value) {
                errors.push(error);
            }
        }

        match response_resource.usage.as_ref() {
            None => errors.push("usage is missing".to_string()),
            Some(usage_resource) => errors.extend(ValidatorResponseResource::validate_usage_resource(usage_resource)),
        }

        match response_resource.output.as_deref() {
            None => errors.push("output is missing".to_string()),
            Some(output) if !ValidatorResponseResource::has_output_text(output) => {
                errors.push("output has no output_text".to_string());
            }
            Some(_) => {}
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                OpenAiResponsesGatewayException::InvalidOpenAiResponse,
                "The model's response is invalid",
                ContextualData::new()
                    .with("resource_name", "response")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }

    fn validate_usage_resource(usage_resource: &UsageResource) -> Vec<String> {
        [
            ("usage.input_tokens", usage_resource.input_tokens),
            ("usage.output_tokens", usage_resource.output_tokens),
        ]
        .into_iter()
        .filter_map(
            |(property_name, property_value): (&str, Option<i64>)| match property_value {
                None => Some(format!("{property_name} is missing")),
                Some(value) if value < 0 => Some(format!("{property_name} is negative")),
                Some(_) => None,
            },
        )
        .collect()
    }

    fn has_output_text(output: &[OutputItemResource]) -> bool {
        output
            .iter()
            .filter(|output_item: &&OutputItemResource| output_item.item_type.as_deref() == Some("message"))
            .flat_map(|output_item: &OutputItemResource| output_item.content.iter().flatten())
            .any(|content: &ContentResource| {
                content.content_type.as_deref() == Some("output_text") && content.text.is_some()
            })
    }
}
