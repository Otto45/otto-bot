import type { Greeting } from "#src/managers/models/greeting.ts";
import type { GreetingResponse } from "#src/services/http/wire_models/greeting_response.ts";

export class GreetingResponseMapper {
  public static toGreetingResponse(greeting: Greeting): GreetingResponse {
    return { message: greeting.message };
  }
}
