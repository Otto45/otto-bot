// A config model: what a configuration source read, before it is validated. Every field can be missing, so the
// validator can say what is wrong with it.
export interface ExampleConfig {
  greetingPrefix?: string | undefined;
}
