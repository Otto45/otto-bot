import { JsonPlaceholderGatewayException } from "#src/managers/exceptions/json_placeholder_gateway_exception.ts";

/** JSON Placeholder answered with a user that doesn't have what a UserProfile needs. */
export class InvalidUserProfileResponseException extends JsonPlaceholderGatewayException {}
