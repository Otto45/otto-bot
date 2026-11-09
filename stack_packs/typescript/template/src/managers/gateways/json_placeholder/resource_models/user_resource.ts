import { z } from "zod";

import { AddressResourceSchema } from "#src/managers/gateways/json_placeholder/resource_models/address_resource.ts";
import { CompanyResourceSchema } from "#src/managers/gateways/json_placeholder/resource_models/company_resource.ts";

export const UserResourceSchema = z.object({
  id: z.int().nullish(),
  name: z.string().nullish(),
  username: z.string().nullish(),
  email: z.string().nullish(),
  phone: z.string().nullish(),
  website: z.string().nullish(),
  address: AddressResourceSchema.nullish(),
  company: CompanyResourceSchema.nullish(),
});

export type UserResource = z.infer<typeof UserResourceSchema>;
