export class ValidatedValue {
  /**
   * A value a validator has checked is there (a mapper's or a settings loader's): it throws only if the validator is
   * wrong, which is a bug, so it is an unexpected Error rather than an ApplicationBaseException.
   */
  public static expect<T>(value: T | null | undefined, validatorName: string): T {
    if (value === null || value === undefined) {
      throw new Error(`validated by ${validatorName}`);
    }
    return value;
  }
}
