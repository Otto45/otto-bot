import { createCsrfMiddleware, createStart } from "@tanstack/react-start";

import { applicationExceptionMiddleware } from "#src/middleware/application_exception_middleware.ts";

// TanStack Start's global middleware. A server function is called only by the app's own pages (CSRF: a request from
// another site is refused), and every exception one throws is logged and told to the browser as the person may see it.
export const startInstance = createStart(() => ({
  requestMiddleware: [createCsrfMiddleware({ filter: (context) => context.handlerType === "serverFn" })],
  functionMiddleware: [applicationExceptionMiddleware],
}));
