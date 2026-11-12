import { createRouter } from "@tanstack/react-router";

import { ErrorPage } from "#src/components/error_page.tsx";
import { NotFoundPage } from "#src/components/not_found_page.tsx";
import { routeTree } from "#src/routeTree.gen.ts";

// The router: the routes TanStack Start generated from src/routes/ (routeTree.gen.ts), and what a page shows when its
// loader fails or what it asked for isn't there. A new router for every request on the server, and one in the browser.
export function getRouter() {
  return createRouter({
    routeTree,
    scrollRestoration: true,
    defaultPreload: "intent",
    defaultErrorComponent: ErrorPage,
    defaultNotFoundComponent: NotFoundPage,
  });
}

// The router's type, for typed links, params and loader data: TanStack Start registers it for its own package
// (routeTree.gen.ts), and TanStack Router reads it from this one.
declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof getRouter>;
  }
}
