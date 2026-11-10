use std::sync::Arc;

use tokio::sync::watch;

use crate::otto_bot_facade::OttoBotFacade;

/// What every request shares, through the router's state: the one facade the service created, the token a request
/// must bear, and whether the service is shutting down (which ends every event stream, so shutdown isn't held open).
#[derive(Clone)]
pub struct HttpServiceState {
    pub otto_bot_facade: Arc<OttoBotFacade>,
    pub api_token: Arc<str>,
    pub shutting_down: watch::Receiver<bool>,
}
