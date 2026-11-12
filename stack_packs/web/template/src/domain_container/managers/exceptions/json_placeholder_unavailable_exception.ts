import { JsonPlaceholderGatewayException } from "#src/domain_container/managers/exceptions/json_placeholder_gateway_exception.ts";

/** JSON Placeholder failed on its side, is limiting requests, or couldn't be reached, through every retry. */
export class JsonPlaceholderUnavailableException extends JsonPlaceholderGatewayException {}
