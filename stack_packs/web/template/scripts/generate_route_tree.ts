// Writes src/routeTree.gen.ts as TanStack Start's own build does, without building: Vite is started with the app's
// configuration (which generates the route tree from src/routes/) and closed again, listening on nothing.
import { createServer } from "vite";
import type { ViteDevServer } from "vite";

const server: ViteDevServer = await createServer({
  server: { middlewareMode: true, hmr: false, watch: null },
  logLevel: "warn",
});
await server.close();
