import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";

// The build: TanStack Start (the routes, server functions and server entry), Nitro (the Node server it runs on), React
// and Tailwind. Import protection keeps the server's code out of the browser: a page or component that imports one of
// these folders, or a `*.server.ts` file, fails the build (in development too), so nothing in them, and no secret they
// read, is ever sent to a browser.
export default defineConfig({
  server: { port: 3000 },
  plugins: [
    tanstackStart({
      importProtection: {
        behavior: "error",
        client: { files: ["**/*.server.*", "**/src/domain_container/**", "**/src/mappers/**"] },
      },
    }),
    nitro({ plugins: ["./src/nitro_plugins/server_entry_loader.ts"] }),
    viteReact(),
    tailwindcss(),
  ],
});
