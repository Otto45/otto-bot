import { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import type { ExceptionFamily } from "#src/managers/exceptions/exception_family.ts";

/** The JsonPlaceholderGateway's exceptions: one family. A new exception is a subclass. */
export class JsonPlaceholderGatewayException extends ApplicationBaseException {
  public readonly family = "JsonPlaceholderGateway" as const;

  public override getFamily(): ExceptionFamily {
    return this;
  }
}
