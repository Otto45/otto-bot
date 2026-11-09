use serde_json::Value;

use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::mappers::api_error_mapper::ApiErrorMapper;
use crate::managers::gateways::openai_responses::resource_models::output_item_resource::{
    OutputItemResource, PatchOperationResource,
};
use crate::managers::gateways::openai_responses::resource_models::response_resource::{
    IncompleteDetailsResource, ResponseResource, UsageResource,
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
            // A response that failed says why; one that failed on the API's side may succeed if sent again.
            if let Some(error_resource) = response_resource.error.as_ref() {
                if let Some(api_error_type) = error_resource.error_type.as_deref() {
                    context.insert("api_error_type", api_error_type);
                }
                if let Some(api_error_code) = error_resource.code.as_deref() {
                    context.insert("api_error_code", api_error_code);
                }
                if ApiErrorMapper::to_exception_kind(
                    None,
                    error_resource.error_type.as_deref(),
                    error_resource.code.as_deref(),
                ) == OpenAiResponsesGatewayException::OpenAiUnavailable
                {
                    return Err(ApplicationBaseException::new(
                        OpenAiResponsesGatewayException::OpenAiUnavailable,
                        "The OpenAI API failed on its side",
                        context,
                    ));
                }
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
            ("model", response_resource.model.as_deref()),
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
            Some(output) => {
                for (index, output_item) in output.iter().enumerate() {
                    errors.extend(ValidatorResponseResource::validate_output_item(output_item, index));
                }
            }
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

    // Only the items the harness acts on must have what it reads; any other item is only sent back.
    fn validate_output_item(output_item: &Value, index: usize) -> Vec<String> {
        let output_item_resource: OutputItemResource =
            match serde_json::from_value::<OutputItemResource>(output_item.clone()) {
                Ok(output_item_resource) => output_item_resource,
                Err(exception) => return vec![format!("output[{index}]: {exception}")],
            };

        let mut errors: Vec<String> = Vec::new();
        match output_item_resource.item_type.as_deref() {
            Some("function_call") => {
                for (property_name, property_value) in [
                    ("call_id", output_item_resource.call_id.as_deref()),
                    ("name", output_item_resource.name.as_deref()),
                ] {
                    if let Some(error) =
                        ValidatorString::validate(&format!("output[{index}].{property_name}"), property_value)
                    {
                        errors.push(error);
                    }
                }
                if output_item_resource.arguments.is_none() {
                    errors.push(format!("output[{index}].arguments is missing"));
                }
            }
            Some("apply_patch_call") => {
                if let Some(error) = ValidatorString::validate(
                    &format!("output[{index}].call_id"),
                    output_item_resource.call_id.as_deref(),
                ) {
                    errors.push(error);
                }
                match output_item_resource.operation.as_ref() {
                    None => errors.push(format!("output[{index}].operation is missing")),
                    Some(operation) => {
                        errors.extend(ValidatorResponseResource::validate_patch_operation(operation, index));
                    }
                }
            }
            None => errors.push(format!("output[{index}].type is missing")),
            Some(_) => {}
        }
        errors
    }

    fn validate_patch_operation(operation: &PatchOperationResource, index: usize) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(error) =
            ValidatorString::validate(&format!("output[{index}].operation.path"), operation.path.as_deref())
        {
            errors.push(error);
        }
        match operation.operation_type.as_deref() {
            Some("create_file") | Some("update_file") => {
                if operation.diff.is_none() {
                    errors.push(format!("output[{index}].operation.diff is missing"));
                }
            }
            Some("delete_file") => {}
            Some(_) => errors.push(format!("output[{index}].operation.type is not a file operation")),
            None => errors.push(format!("output[{index}].operation.type is missing")),
        }
        errors
    }
}
