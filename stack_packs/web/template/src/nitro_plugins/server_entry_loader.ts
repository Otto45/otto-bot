import { definePlugin } from "nitro";
import { fetchService } from "nitro/services";

// Nitro loads the server entry (src/server.ts) when the first request comes; this plugin loads it as the server starts
// (Nitro holds requests until it is done), so the facade is created, and invalid configuration ends the process, before
// any request is served. Loading it is asking it something: whether the app is up (src/routes/health.ts).
export default definePlugin(async (): Promise<void> => {
  await fetchService("ssr", "http://localhost/health");
});
