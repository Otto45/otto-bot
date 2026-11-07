use crate::managers::gateways::json_placeholder::resource_models::user_resource::{
    AddressResource, CompanyResource, UserResource,
};
use crate::managers::models::user_profile::UserProfile;

pub struct UserProfileMapper;

impl UserProfileMapper {
    /// Expects a resource that has already passed ValidatorUserResource: each `expect` states a checked fact, and
    /// fails only if the validator is wrong.
    pub fn to_user_profile(user_resource: UserResource) -> UserProfile {
        let address_resource: AddressResource = user_resource.address.expect("validated by ValidatorUserResource");
        let company_resource: CompanyResource = user_resource.company.expect("validated by ValidatorUserResource");

        UserProfile {
            id: user_resource.id.expect("validated by ValidatorUserResource"),
            name: user_resource.name.expect("validated by ValidatorUserResource"),
            username: user_resource.username.expect("validated by ValidatorUserResource"),
            email: user_resource.email.expect("validated by ValidatorUserResource"),
            city: address_resource.city.expect("validated by ValidatorUserResource"),
            company_name: company_resource.name.expect("validated by ValidatorUserResource"),
        }
    }
}
