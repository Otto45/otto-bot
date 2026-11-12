import type { Greeting } from "#src/domain_container/managers/models/greeting.ts";
import type { GreetingResponse } from "#src/wire_models/greeting_response.ts";

export class GreetingResponseMapper {
  public static toGreetingResponse(greeting: Greeting): GreetingResponse {
    return { message: greeting.message };
  }
}
