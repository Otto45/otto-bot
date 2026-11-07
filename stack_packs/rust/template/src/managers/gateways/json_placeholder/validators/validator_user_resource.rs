use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::json_placeholder_gateway_exception::JsonPlaceholderGatewayException;
use crate::managers::gateways::json_placeholder::resource_models::user_resource::{
    AddressResource, CompanyResource, UserResource,
};

pub struct ValidatorUserResource;

impl ValidatorUserResource {
    pub fn validate_user_resource(user_resource: &UserResource) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();

        match user_resource.id {
            None => errors.push("id is missing".to_string()),
            Some(id) if id <= 0 => errors.push("id is not positive".to_string()),
            Some(_) => {}
        }

        for (property_name, property_value) in [
            ("name", user_resource.name.as_deref()),
            ("username", user_resource.username.as_deref()),
            ("email", user_resource.email.as_deref()),
        ] {
            if let Some(error) = ValidatorString::validate(property_name, property_value) {
                errors.push(error);
            }
        }

        errors.extend(ValidatorUserResource::validate_address_resource(
            user_resource.address.as_ref(),
        ));
        errors.extend(ValidatorUserResource::validate_company_resource(
            user_resource.company.as_ref(),
        ));

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                JsonPlaceholderGatewayException::InvalidUserProfileResponse,
                "User profile response is invalid",
                ContextualData::new()
                    .with("resource_name", "user")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }

    fn validate_address_resource(address_resource: Option<&AddressResource>) -> Option<String> {
        match address_resource {
            None => Some("address is missing".to_string()),
            Some(address_resource) => ValidatorString::validate("address.city", address_resource.city.as_deref()),
        }
    }

    fn validate_company_resource(company_resource: Option<&CompanyResource>) -> Option<String> {
        match company_resource {
            None => Some("company is missing".to_string()),
            Some(company_resource) => ValidatorString::validate("company.name", company_resource.name.as_deref()),
        }
    }
}
