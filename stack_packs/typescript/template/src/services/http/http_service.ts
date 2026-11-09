import { serve } from "@hono/node-server";
import type { ServerType } from "@hono/node-server";
import { OpenAPIHono } from "@hono/zod-openapi";
import { HTTPException } from "hono/http-exception";

import { ExampleAppFacade } from "#src/example_app_facade.ts";
import { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { ApplicationLogger } from "#src/managers/logger/application_logger.ts";
import { ApplicationExceptionHandler } from "#src/services/http/exception_handlers/application_exception_handler.ts";
import { RequestRejectionHandler } from "#src/services/http/exception_handlers/request_rejection_handler.ts";
import { UnexpectedErrorHandler } from "#src/services/http/exception_handlers/unexpected_error_handler.ts";
import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import { RequestRejectionMapper } from "#src/services/http/mappers/request_rejection_mapper.ts";
import { CustomerRouter } from "#src/services/http/routers/customer_router.ts";
import { UserProfileRouter } from "#src/services/http/routers/user_profile_router.ts";
import { ShutdownSignal } from "#src/services/http/shutdown_signal.ts";

const HTTP_UNPROCESSABLE_CONTENT = 422 as const;

// A service runs and listens. It sits outside the domain container and holds no application logic: it works the way its
// framework does, hands the Domain Facade only domain models, maps the domain models it gets back to response models,
// and translates the facade's exceptions into HTTP errors (ApplicationExceptionHandler).
export class HttpService {
  /**
   * Creates the Domain Facade, serves requests with it until Ctrl-C or SIGTERM (finishing the requests under way), then
   * closes it. The facade is created once and lives as long as the server: every request gets this one instance from
   * its context, and it is closed, with everything the locator cached, when the server has stopped, whether serving
   * failed or not.
   */
  public static async serve(host: string, port: number): Promise<void> {
    // e.g. invalid configuration, which fails while the facade is created, before the server listens
    await using exampleAppFacade: ExampleAppFacade = new ExampleAppFacade();

    const server: ServerType = await HttpService.listen(HttpService.getApplication(exampleAppFacade), host, port);
    ApplicationLogger.info(
      "The service is listening",
      new ContextualData().withValue("host", host).withValue("port", port),
    );
    await ShutdownSignal.received();
    await HttpService.close(server);
  }

  /** The application; without a facade, only for its contract (it can't answer requests). */
  public static getApplication(exampleAppFacade: ExampleAppFacade | undefined): OpenAPIHono<HttpEnvironment> {
    const application: OpenAPIHono<HttpEnvironment> = new OpenAPIHono<HttpEnvironment>({
      defaultHook: (result, context): Response | undefined => {
        if (!result.success) {
          return context.json(RequestRejectionMapper.toErrorResponse(result.error), HTTP_UNPROCESSABLE_CONTENT);
        }
        return undefined;
      },
    });
    if (exampleAppFacade !== undefined) {
      application.use(async (context, next): Promise<void> => {
        context.set("exampleAppFacade", exampleAppFacade);
        await next();
      });
    }
    CustomerRouter.register(application);
    UserProfileRouter.register(application);
    application.onError((error, context): Response => {
      if (error instanceof ApplicationBaseException) {
        return ApplicationExceptionHandler.handle(error, context);
      }
      if (error instanceof HTTPException) {
        return RequestRejectionHandler.handle(error, context);
      }
      return UnexpectedErrorHandler.handle(error, context);
    });
    return application;
  }

  /** The service's contract, built from the routers. Building it creates no facade, so it needs no configuration. */
  public static getContract(): object {
    return HttpService.getApplication(undefined).getOpenAPI31Document({
      openapi: "3.1.0",
      info: { title: "Example App", version: "0.1.0" },
    });
  }

  // The server once it listens; a port that can't be listened on fails here.
  private static listen(application: OpenAPIHono<HttpEnvironment>, host: string, port: number): Promise<ServerType> {
    return new Promise<ServerType>((resolve: (server: ServerType) => void, reject: (error: Error) => void): void => {
      const server: ServerType = serve({ fetch: application.fetch, hostname: host, port }, (): void => {
        resolve(server);
      });
      server.once("error", reject);
    });
  }

  // Stops taking connections and waits for the requests under way to finish; idle connections are closed.
  private static close(server: ServerType): Promise<void> {
    return new Promise<void>((resolve: () => void, reject: (error: Error) => void): void => {
      server.close((error: Error | undefined): void => {
        if (error === undefined) {
          resolve();
        } else {
          reject(error);
        }
      });
    });
  }
}
