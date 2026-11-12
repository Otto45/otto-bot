import { z } from "zod";

export const UserProfileRequestSchema = z.object({ userId: z.int() });

export type UserProfileRequest = z.infer<typeof UserProfileRequestSchema>;
