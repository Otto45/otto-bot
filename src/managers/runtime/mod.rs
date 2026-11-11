//! What a manager owns while the app runs: live sessions, their runs and their event channels, and the locks its runs
//! hold. Not domain models: they hold tasks, channels and open files, and change as runs start and end. Only their
//! manager uses them, and its close() releases them.
pub mod held_lock;
pub mod live_run;
pub mod live_session;
pub mod session_event_log;
