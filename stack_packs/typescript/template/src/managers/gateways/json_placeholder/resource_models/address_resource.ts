import { z } from "zod";

// Resource models: the JSON body read through a zod schema as it is, with every field optional (missing or null) so
// that anything the API sends is read and the validator can say what is wrong with it. Fields the schema doesn't name
// are dropped.
export const AddressResourceSchema = z.object({
  street: z.string().nullish(),
  suite: z.string().nullish(),
  city: z.string().nullish(),
  zipcode: z.string().nullish(),
});

export type AddressResource = z.infer<typeof AddressResourceSchema>;
