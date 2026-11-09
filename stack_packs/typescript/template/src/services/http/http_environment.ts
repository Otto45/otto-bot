import type { ExampleAppFacade } from "#src/example_app_facade.ts";

// What a request's context holds: the one facade the service created (HttpService.serve), set on every request.
export interface HttpEnvironment {
  Variables: {
    exampleAppFacade: ExampleAppFacade;
  };
}
