from typing import final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_user_profile_response_exception import (
    InvalidUserProfileResponseException,
)
from example_app.managers.gateways.json_placeholder.resource_models.address_resource import AddressResource
from example_app.managers.gateways.json_placeholder.resource_models.company_resource import CompanyResource
from example_app.managers.gateways.json_placeholder.resource_models.user_resource import UserResource


@final
class ValidatorUserResource:
    @staticmethod
    def validate_user_resource(user_resource: UserResource) -> None:
        errors: list[str] = []

        if user_resource.id is None:
            errors.append("id is missing")
        elif user_resource.id <= 0:
            errors.append("id is not positive")

        for property_name, property_value in (
            ("name", user_resource.name),
            ("username", user_resource.username),
            ("email", user_resource.email),
        ):
            error: str | None = ValidatorString.validate(property_name, property_value)
            if error is not None:
                errors.append(error)

        for nested_error in (
            ValidatorUserResource._validate_address_resource(user_resource.address),
            ValidatorUserResource._validate_company_resource(user_resource.company),
        ):
            if nested_error is not None:
                errors.append(nested_error)

        if errors:
            raise InvalidUserProfileResponseException(
                "User profile response is invalid",
                ContextualData().with_value("resource_name", "user").with_value("errors", "; ".join(errors)),
            )

    @staticmethod
    def _validate_address_resource(address_resource: AddressResource | None) -> str | None:
        if address_resource is None:
            return "address is missing"
        return ValidatorString.validate("address.city", address_resource.city)

    @staticmethod
    def _validate_company_resource(company_resource: CompanyResource | None) -> str | None:
        if company_resource is None:
            return "company is missing"
        return ValidatorString.validate("company.name", company_resource.name)
