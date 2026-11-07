// What a manager owns while the app runs: live sessions, their runs and their event channels. Not domain models:
// they hold tasks and channels, and change as runs start and end. Only their manager uses them, and its close()
// releases them.
pub mod live_run;
pub mod live_session;
