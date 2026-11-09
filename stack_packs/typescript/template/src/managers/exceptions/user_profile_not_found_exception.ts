import { JsonPlaceholderGatewayException } from "#src/managers/exceptions/json_placeholder_gateway_exception.ts";

/** No user profile has the id asked for: the caller can put it right, so its message is written for the person using the app. */
export class UserProfileNotFoundException extends JsonPlaceholderGatewayException {}
