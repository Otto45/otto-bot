import type { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import type { ExceptionFamily } from "#src/managers/exceptions/exception_family.ts";

// The base of every exception the application raises: a message and a context. JavaScript keeps the stack where it was
// thrown (`stack`) and the error it was thrown from (`cause`), and both are logged with it.
//
// Each component's exceptions are a family: a class that extends this one (e.g. RelationalDatabaseGatewayException),
// with a subclass per exception (e.g. CustomerNotFoundException). Each family names itself in `family`, a literal type
// of its own, and getFamily() returns the exception as a member of ExceptionFamily, the union of every family, so a
// switch on `exception.getFamily().family` that names every family is checked by the compiler: a new family fails to
// compile until it is added to the union, and then every such switch fails until it decides the new family.
export abstract class ApplicationBaseException extends Error {
  public readonly context: ContextualData;

  public constructor(message: string, context: ContextualData, options?: ErrorOptions) {
    super(message, options);
    this.context = context;
    // The name it is logged under: its class's (e.g. "CustomerNotFoundException").
    this.name = new.target.name;
  }

  public abstract getFamily(): ExceptionFamily;
}
