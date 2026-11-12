import { ValidatorString } from "#src/domain_container/managers/common/validators/validator_string.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidUserProfileResponseException } from "#src/domain_container/managers/exceptions/invalid_user_profile_response_exception.ts";
import type { AddressResource } from "#src/domain_container/managers/gateways/json_placeholder/resource_models/address_resource.ts";
import type { CompanyResource } from "#src/domain_container/managers/gateways/json_placeholder/resource_models/company_resource.ts";
import type { UserResource } from "#src/domain_container/managers/gateways/json_placeholder/resource_models/user_resource.ts";

export class ValidatorUserResource {
  public static validateUserResource(userResource: UserResource): void {
    const errors: string[] = [];

    if (userResource.id === null || userResource.id === undefined) {
      errors.push("id is missing");
    } else if (userResource.id <= 0) {
      errors.push("id is not positive");
    }

    for (const error of [
      ValidatorString.validate("name", userResource.name),
      ValidatorString.validate("username", userResource.username),
      ValidatorString.validate("email", userResource.email),
      ValidatorUserResource.validateAddressResource(userResource.address),
      ValidatorUserResource.validateCompanyResource(userResource.company),
    ]) {
      if (error !== undefined) {
        errors.push(error);
      }
    }

    if (errors.length > 0) {
      throw new InvalidUserProfileResponseException(
        "User profile response is invalid",
        new ContextualData().withValue("resource_name", "user").withValue("errors", errors.join("; ")),
      );
    }
  }

  private static validateAddressResource(addressResource: AddressResource | null | undefined): string | undefined {
    if (addressResource === null || addressResource === undefined) {
      return "address is missing";
    }
    return ValidatorString.validate("address.city", addressResource.city);
  }

  private static validateCompanyResource(companyResource: CompanyResource | null | undefined): string | undefined {
    if (companyResource === null || companyResource === undefined) {
      return "company is missing";
    }
    return ValidatorString.validate("company.name", companyResource.name);
  }
}
