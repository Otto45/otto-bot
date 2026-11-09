import type { ContextValue } from "#src/managers/exceptions/context_value.ts";

// What an exception says about where it happened, under snake_case keys, for the engineers who read the logs. It never
// holds anything sensitive: no bound parameter values, no credentials. It never changes once made: withValue returns a
// new one, so a context is built in one expression: `new ContextualData().withValue("customer_id", customerId)`.
export class ContextualData {
  private readonly values: ReadonlyMap<string, ContextValue>;

  public constructor(values: ReadonlyMap<string, ContextValue> = new Map<string, ContextValue>()) {
    this.values = values;
  }

  public withValue(key: string, value: ContextValue): ContextualData {
    const values: Map<string, ContextValue> = new Map<string, ContextValue>(this.values);
    values.set(key, value);
    return new ContextualData(values);
  }

  public get(key: string): ContextValue | undefined {
    return this.values.get(key);
  }

  /** The values, sorted by key, so the same context is always written the same way. */
  public toRecord(): Record<string, ContextValue> {
    const keys: string[] = [...this.values.keys()].sort();
    const record: Record<string, ContextValue> = {};
    for (const key of keys) {
      const value: ContextValue | undefined = this.values.get(key);
      if (value !== undefined) {
        record[key] = value;
      }
    }
    return record;
  }
}
