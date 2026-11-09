// A stateless helper shared by every validator: a class with static methods only, never built and never registered in
// the ServiceLocator.
export class ValidatorString {
  public static validate(propertyName: string, propertyValue: string | null | undefined): string | undefined {
    if (propertyValue === null || propertyValue === undefined) {
      return `${propertyName} is missing`;
    }
    if (propertyValue === "") {
      return `${propertyName} is empty`;
    }
    if (propertyValue.trim() === "") {
      return `${propertyName} is whitespace`;
    }
    return undefined;
  }
}
