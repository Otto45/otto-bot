// A value an exception's context may hold: primitives only, so a model, a row or a bound parameter's value cannot be
// put in a context at all.
export type ContextValue = string | number | boolean;
