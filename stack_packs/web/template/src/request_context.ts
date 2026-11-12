import type { ExampleAppFacade } from "#src/domain_container/example_app_facade.ts";

// What every request on the server carries, handed in by the server entry (src/server.ts): the one facade the server
// created. Server functions and middleware read it from their `context`.
export interface RequestContext {
  readonly exampleAppFacade: ExampleAppFacade;
}
