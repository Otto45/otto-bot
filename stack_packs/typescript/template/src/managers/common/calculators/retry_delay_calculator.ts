// How long a gateway waits before sending a failed request again: doubling from the first, up to the longest, with up
// to a quarter of a second added at random so requests that failed together aren't sent together. The longest a
// service's own Retry-After is waited for; asking for longer is treated as this.
const FIRST_RETRY_DELAY_MILLISECONDS: number = 1_000;
const LONGEST_RETRY_DELAY_MILLISECONDS: number = 20_000;
const MAX_RETRY_JITTER_MILLISECONDS: number = 250;
const LONGEST_RETRY_AFTER_MILLISECONDS: number = 60_000;

// The waits between a gateway's retries, the same for every gateway: a gateway that retries keeps its own loop (it
// decides on its own exceptions) and asks this how long to wait.
export class RetryDelayCalculator {
  /**
   * The retry-th wait (the first retry's is 1): what the service asked for (retryAfterSeconds, from its Retry-After, up
   * to a minute), or doubling from a second up to 20, whichever is longer, with jitter.
   */
  public static getRetryDelayMilliseconds(retry: number, retryAfterSeconds: number | undefined): number {
    const backoff: number = Math.min(
      FIRST_RETRY_DELAY_MILLISECONDS * 2 ** (retry - 1),
      LONGEST_RETRY_DELAY_MILLISECONDS,
    );
    const asked: number =
      retryAfterSeconds === undefined ? 0 : Math.min(retryAfterSeconds * 1000, LONGEST_RETRY_AFTER_MILLISECONDS);
    return Math.max(backoff, asked) + Math.random() * MAX_RETRY_JITTER_MILLISECONDS;
  }

  /**
   * A Retry-After header's wait, in whole seconds; undefined for one that isn't a number of seconds (an HTTP date, which
   * is then waited out by the backoff instead).
   */
  public static parseRetryAfter(retryAfter: string | null): number | undefined {
    const trimmed: string = retryAfter?.trim() ?? "";
    return /^\d+$/.test(trimmed) ? Number(trimmed) : undefined;
  }
}
