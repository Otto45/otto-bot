/// Which of a session's past events a client subscribing gets before the ones to come. Only the current run's events
/// (or the last run's, between runs) are kept, from its `RunStarted` on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEventReplay {
    /// None: only the events to come (a client that sends its requests itself, after subscribing).
    FromNow,
    /// The current run's events from its start, so a client that subscribes just after sending a request misses none.
    FromRunStart,
    /// The kept events after the one with this id: what a client that lost its connection missed.
    After(u64),
}
