// How a server function failed, as the browser is told: something the person can put right (Invalid: the exception's
// own message), something that isn't there (NotFound: the page shows its not-found state), something unavailable for
// now (Unavailable: they can try again later), or a failure only an engineer can put right (Failed). A const object and
// its values' type, as TypeScript's enums aren't plain JavaScript.
export const ServerFunctionFailure = {
  Invalid: "Invalid",
  NotFound: "NotFound",
  Unavailable: "Unavailable",
  Failed: "Failed",
} as const;

export type ServerFunctionFailure = (typeof ServerFunctionFailure)[keyof typeof ServerFunctionFailure];
