// A response model: what a server function sends the browser, which only the app's own pages read. Every field is
// readonly, and holds what the browser can carry as it is (strings, numbers, booleans, arrays and objects of them): a
// date is its ISO 8601 text, an amount of money its minor units and currency.
export interface GreetingResponse {
  readonly message: string;
}
