import type { ApplicationBaseException } from "#src/domain_container/managers/exceptions/application_base_exception.ts";
import { CustomerNotFoundException } from "#src/domain_container/managers/exceptions/customer_not_found_exception.ts";
import type { ExceptionFamily } from "#src/domain_container/managers/exceptions/exception_family.ts";
import { JsonPlaceholderUnavailableException } from "#src/domain_container/managers/exceptions/json_placeholder_unavailable_exception.ts";
import { UserProfileNotFoundException } from "#src/domain_container/managers/exceptions/user_profile_not_found_exception.ts";
import { ServerFunctionFailure } from "#src/mappers/server_function_failure.ts";

const NOT_FOUND_MESSAGE: string = "We couldn't find that.";
const UNAVAILABLE_MESSAGE: string = "This isn't available right now. Please try again in a moment.";
const FAILED_MESSAGE: string = "Something went wrong on our side. Please try again.";

// Translates the domain container's exceptions into what the browser is told. The browser receives only an error's
// message (never its stack, context or cause), and only an Invalid exception's own message: its message is written for
// the person using the app, while every other one describes internals (tables, SQL, upstream services), which are
// logged instead.
export class ServerFunctionErrorMapper {
  public static toFailure(exception: ApplicationBaseException): ServerFunctionFailure {
    const family: ExceptionFamily = exception.getFamily();
    // Every family is named, with no `default`, so a new one doesn't compile until it is decided here.
    switch (family.family) {
      case "InvalidCustomer":
        return ServerFunctionFailure.Invalid;
      case "RelationalDatabaseGateway":
        return family instanceof CustomerNotFoundException
          ? ServerFunctionFailure.NotFound
          : ServerFunctionFailure.Failed;
      case "JsonPlaceholderGateway":
        if (family instanceof UserProfileNotFoundException) {
          return ServerFunctionFailure.NotFound;
        }
        // An upstream service down for now, through every retry: the person can try again later.
        return family instanceof JsonPlaceholderUnavailableException
          ? ServerFunctionFailure.Unavailable
          : ServerFunctionFailure.Failed;
      case "ConfigurationProvider":
      case "DocumentDatabaseGateway":
        return ServerFunctionFailure.Failed;
    }
  }

  /**
   * What the browser is told of a failure, as an error's message. A NotFound failure is thrown as the router's
   * notFound() instead, so a page shows its not-found state.
   */
  public static toBrowserMessage(exception: ApplicationBaseException, failure: ServerFunctionFailure): string {
    switch (failure) {
      case ServerFunctionFailure.Invalid:
        return exception.message;
      case ServerFunctionFailure.NotFound:
        return NOT_FOUND_MESSAGE;
      case ServerFunctionFailure.Unavailable:
        return UNAVAILABLE_MESSAGE;
      case ServerFunctionFailure.Failed:
        return FAILED_MESSAGE;
    }
  }

  /** What the browser is told of an error that is no application exception (a bug, a library's error). */
  public static toUnexpectedBrowserMessage(): string {
    return FAILED_MESSAGE;
  }
}
