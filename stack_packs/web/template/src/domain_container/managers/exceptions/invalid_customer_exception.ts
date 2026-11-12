import { ApplicationBaseException } from "#src/domain_container/managers/exceptions/application_base_exception.ts";
import type { ExceptionFamily } from "#src/domain_container/managers/exceptions/exception_family.ts";

/** A family of one. Raised for something the caller can put right, so its message is written for the person using the app: a service answers with it as it is. What only engineers need goes in the context, which is logged. */
export class InvalidCustomerException extends ApplicationBaseException {
  public readonly family = "InvalidCustomer" as const;

  public override getFamily(): ExceptionFamily {
    return this;
  }
}
