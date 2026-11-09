import { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import type { ExceptionFamily } from "#src/managers/exceptions/exception_family.ts";

/** The RelationalDatabaseGateway's exceptions: one family. A new exception is a subclass. */
export class RelationalDatabaseGatewayException extends ApplicationBaseException {
  public readonly family = "RelationalDatabaseGateway" as const;

  public override getFamily(): ExceptionFamily {
    return this;
  }
}
