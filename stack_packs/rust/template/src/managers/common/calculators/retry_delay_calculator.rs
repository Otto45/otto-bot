use std::time::Duration;

// How long a gateway waits before sending a failed request again: doubling from the first, up to the longest, with up
// to a quarter of a second added at random so requests that failed together aren't sent together. The longest a
// service's own Retry-After is waited for; asking for longer is treated as this.
const FIRST_RETRY_DELAY: Duration = Duration::from_secs(1);
const LONGEST_RETRY_DELAY: Duration = Duration::from_secs(20);
const MAX_RETRY_JITTER_MILLISECONDS: u128 = 250;
const LONGEST_RETRY_AFTER: Duration = Duration::from_secs(60);

/// The waits between a gateway's retries, the same for every gateway: a gateway that retries keeps its own loop (it
/// decides on its own exceptions) and asks this how long to wait.
pub struct RetryDelayCalculator;

impl RetryDelayCalculator {
    /// The `retry`th wait (the first retry's is 1): what the service asked for (`retry_after_seconds`, from its
    /// Retry-After, up to a minute), or doubling from a second up to 20, whichever is longer, with jitter.
    pub fn get_retry_delay(retry: i64, retry_after_seconds: Option<i64>) -> Duration {
        let doubling: u32 = u32::try_from(retry.saturating_sub(1)).unwrap_or(u32::MAX).min(16);
        let backoff: Duration = FIRST_RETRY_DELAY
            .saturating_mul(2_u32.saturating_pow(doubling))
            .min(LONGEST_RETRY_DELAY);
        let retry_after: Duration = retry_after_seconds
            .and_then(|seconds: i64| u64::try_from(seconds).ok())
            .map_or(Duration::ZERO, Duration::from_secs)
            .min(LONGEST_RETRY_AFTER);
        let jitter: Duration = Duration::from_millis(
            u64::try_from(uuid::Uuid::new_v4().as_u128() % MAX_RETRY_JITTER_MILLISECONDS).unwrap_or(0),
        );
        backoff.max(retry_after) + jitter
    }

    /// A Retry-After header's wait, in whole seconds; None for one that isn't a number of seconds (an HTTP date, which
    /// is then waited out by the backoff instead).
    pub fn parse_retry_after(retry_after: &str) -> Option<i64> {
        retry_after
            .trim()
            .parse::<i64>()
            .ok()
            .filter(|seconds: &i64| *seconds >= 0)
    }
}
