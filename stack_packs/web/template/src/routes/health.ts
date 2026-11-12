import { createFileRoute } from "@tanstack/react-router";

// A server route: an HTTP endpoint for callers outside the app (here a deployment's health check), answered on the
// server with a Response. The app's own pages call server functions instead.
export const Route = createFileRoute("/health")({
  server: {
    handlers: {
      GET: () => Response.json({ status: "ok" }),
    },
  },
});
