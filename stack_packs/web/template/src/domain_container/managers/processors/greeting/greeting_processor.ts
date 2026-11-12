import type { Greeting } from "#src/domain_container/managers/models/greeting.ts";

export class GreetingProcessor {
  private readonly greetingPrefix: string;

  public constructor(greetingPrefix: string) {
    this.greetingPrefix = greetingPrefix;
  }

  public greet(name: string): Greeting {
    return { message: `${this.greetingPrefix}, ${name}!` };
  }
}
