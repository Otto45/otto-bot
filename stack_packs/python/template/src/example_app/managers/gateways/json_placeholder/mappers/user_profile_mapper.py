from typing import Final, final

from example_app.managers.common.validators.validated_value import ValidatedValue
from example_app.managers.gateways.json_placeholder.resource_models.address_resource import AddressResource
from example_app.managers.gateways.json_placeholder.resource_models.company_resource import CompanyResource
from example_app.managers.gateways.json_placeholder.resource_models.user_resource import UserResource
from example_app.managers.models.user_profile import UserProfile

_VALIDATOR_NAME: Final[str] = "ValidatorUserResource"


@final
class UserProfileMapper:
    @staticmethod
    def to_user_profile(user_resource: UserResource) -> UserProfile:
        """Expects a resource that has already passed ValidatorUserResource: each ValidatedValue.expect states a
        checked fact, and fails only if the validator is wrong."""
        address_resource: AddressResource = ValidatedValue.expect(user_resource.address, _VALIDATOR_NAME)
        company_resource: CompanyResource = ValidatedValue.expect(user_resource.company, _VALIDATOR_NAME)

        return UserProfile(
            id=ValidatedValue.expect(user_resource.id, _VALIDATOR_NAME),
            name=ValidatedValue.expect(user_resource.name, _VALIDATOR_NAME),
            username=ValidatedValue.expect(user_resource.username, _VALIDATOR_NAME),
            email=ValidatedValue.expect(user_resource.email, _VALIDATOR_NAME),
            city=ValidatedValue.expect(address_resource.city, _VALIDATOR_NAME),
            company_name=ValidatedValue.expect(company_resource.name, _VALIDATOR_NAME),
        )
