import type { Context, Hono } from "hono";
import type { RouterRoute } from "hono/types";

import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import type { ErrorResponse } from "#src/services/http/wire_models/error_response.ts";

const HTTP_NOT_FOUND = 404 as const;
const HTTP_METHOD_NOT_ALLOWED = 405 as const;
// What a middleware is registered under (`use`): it matches every method, so it is no route's.
const ALL_METHODS: string = "ALL";

// What Hono answers a request no route takes, in the ErrorResponse every other error is answered with, so that callers
// handle one shape of error; Hono's own answer is plain text. A path some route takes by another method is a 405 with
// the methods it takes in `Allow` (Hono has no 405 of its own, so its router is asked), any other a 404.
export class UnmatchedRequestHandler {
  public static handle(application: Hono<HttpEnvironment>, context: Context<HttpEnvironment>): Response {
    const allowedMethods: string[] = UnmatchedRequestHandler.getAllowedMethods(application, context.req.path);
    if (allowedMethods.length === 0) {
      const errorResponse: ErrorResponse = { error: "Not Found" };
      return context.json(errorResponse, HTTP_NOT_FOUND);
    }
    const errorResponse: ErrorResponse = { error: "Method Not Allowed" };
    return context.json(errorResponse, HTTP_METHOD_NOT_ALLOWED, { Allow: allowedMethods.join(", ") });
  }

  // The methods of the routes that take this path, sorted.
  private static getAllowedMethods(application: Hono<HttpEnvironment>, path: string): string[] {
    const methods: Set<string> = new Set<string>(
      application.routes
        .map((route: RouterRoute): string => route.method)
        .filter((method: string): boolean => method !== ALL_METHODS),
    );
    return [...methods]
      .filter((method: string): boolean =>
        application.router.match(method, path)[0].some(([[, route]]) => route.method === method),
      )
      .sort();
  }
}
