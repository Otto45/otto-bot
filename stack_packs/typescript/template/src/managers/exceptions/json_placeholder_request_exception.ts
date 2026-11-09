import { JsonPlaceholderGatewayException } from "#src/managers/exceptions/json_placeholder_gateway_exception.ts";

/** A request to JSON Placeholder failed: it could not be sent, timed out, or was answered with an error status. */
export class JsonPlaceholderRequestException extends JsonPlaceholderGatewayException {}
