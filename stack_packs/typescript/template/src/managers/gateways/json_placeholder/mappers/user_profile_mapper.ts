import { ValidatedValue } from "#src/managers/common/validators/validated_value.ts";
import type { AddressResource } from "#src/managers/gateways/json_placeholder/resource_models/address_resource.ts";
import type { CompanyResource } from "#src/managers/gateways/json_placeholder/resource_models/company_resource.ts";
import type { UserResource } from "#src/managers/gateways/json_placeholder/resource_models/user_resource.ts";
import type { UserProfile } from "#src/managers/models/user_profile.ts";

const VALIDATOR_NAME: string = "ValidatorUserResource";

export class UserProfileMapper {
  /**
   * Expects a resource that has already passed ValidatorUserResource: each ValidatedValue.expect states a checked
   * fact, and fails only if the validator is wrong.
   */
  public static toUserProfile(userResource: UserResource): UserProfile {
    const addressResource: AddressResource = ValidatedValue.expect(userResource.address, VALIDATOR_NAME);
    const companyResource: CompanyResource = ValidatedValue.expect(userResource.company, VALIDATOR_NAME);

    return {
      id: ValidatedValue.expect(userResource.id, VALIDATOR_NAME),
      name: ValidatedValue.expect(userResource.name, VALIDATOR_NAME),
      username: ValidatedValue.expect(userResource.username, VALIDATOR_NAME),
      email: ValidatedValue.expect(userResource.email, VALIDATOR_NAME),
      city: ValidatedValue.expect(addressResource.city, VALIDATOR_NAME),
      companyName: ValidatedValue.expect(companyResource.name, VALIDATOR_NAME),
    };
  }
}
