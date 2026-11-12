import { z } from "zod";

export const CompanyResourceSchema = z.object({
  name: z.string().nullish(),
});

export type CompanyResource = z.infer<typeof CompanyResourceSchema>;
