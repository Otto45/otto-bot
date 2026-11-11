use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::watch;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityRequirement, SecurityScheme};
use utoipa::openapi::{ComponentsBuilder, Info, OpenApi, OpenApiBuilder};
use utoipa_axum::router::OpenApiRouter;

use crate::otto_bot_facade::OttoBotFacade;
use crate::services::http::exception_handlers::unmatched_request_handler::UnmatchedRequestHandler;
use crate::services::http::http_service_state::HttpServiceState;
use crate::services::http::middleware::bearer_token_middleware::BearerTokenMiddleware;
use crate::services::http::middleware::unexpected_error_middleware::UnexpectedErrorMiddleware;
use crate::services::http::routers::app_router::AppRouter;
use crate::services::http::routers::health_router::HealthRouter;
use crate::services::http::routers::session_router::SessionRouter;

// The bearer token's scheme in the contract.
const BEARER_SCHEME: &str = "bearer";

// The HTTP API, otto-bot's second client: the software factory platform starts apps and holds sessions with it. A
// service runs and listens; it sits outside the domain container and holds no application logic: it works the way
// axum does, hands the Domain Facade only domain models (and names: apps are addressed by name, never by path), maps
// what it gets back to response models, and translates the facade's exceptions into HTTP errors.
pub struct HttpService;

impl HttpService {
    /// Creates the Domain Facade, serves requests with it until `shutdown` completes, then closes it. Every request but
    /// /health must bear `api_token`. On shutdown every event stream ends, the requests under way finish, and the
    /// facade is closed (stopping any run under way), whether serving failed or not.
    pub async fn serve(
        listener: TcpListener,
        api_token: String,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // e.g. invalid configuration, which fails while the facade is created
        let otto_bot_facade: Arc<OttoBotFacade> = Arc::new(OttoBotFacade::new()?);
        let (stop, shutting_down): (watch::Sender<bool>, watch::Receiver<bool>) = watch::channel(false);
        let http_service_state: HttpServiceState = HttpServiceState {
            otto_bot_facade: otto_bot_facade.clone(),
            api_token: Arc::from(api_token),
            shutting_down,
        };

        let served: std::io::Result<()> = axum::serve(listener, HttpService::get_application(http_service_state))
            .with_graceful_shutdown(async move {
                shutdown.await;
                // No stream may be open.
                stop.send(true).ok();
            })
            .await;

        otto_bot_facade.close().await;
        Ok(served?)
    }

    pub fn get_application(http_service_state: HttpServiceState) -> Router {
        let (authenticated, _): (Router<HttpServiceState>, OpenApi) = HttpService::build().split_for_parts();
        let (open, _): (Router<HttpServiceState>, OpenApi) = HealthRouter::router().split_for_parts();
        authenticated
            // On the routes only, so a path that is no route's is a 404, token or not.
            .route_layer(axum::middleware::from_fn_with_state(
                http_service_state.clone(),
                BearerTokenMiddleware::handle,
            ))
            .merge(open)
            .fallback(UnmatchedRequestHandler::handle_not_found)
            .method_not_allowed_fallback(UnmatchedRequestHandler::handle_method_not_allowed)
            .layer(axum::middleware::from_fn(UnexpectedErrorMiddleware::handle))
            .with_state(http_service_state)
    }

    /// The service's contract, built from the routers. Building it creates no facade, so it needs no configuration.
    pub fn get_contract() -> OpenApi {
        HttpService::build().merge(HealthRouter::router()).into_openapi()
    }

    // Every route that needs the token, under a contract that says so.
    fn build() -> OpenApiRouter<HttpServiceState> {
        let contract: OpenApi = OpenApiBuilder::new()
            .info(Info::new("otto-bot", env!("CARGO_PKG_VERSION")))
            .components(Some(
                ComponentsBuilder::new()
                    .security_scheme(BEARER_SCHEME, SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
                    .build(),
            ))
            .security(Some(vec![SecurityRequirement::new(
                BEARER_SCHEME,
                Vec::<String>::new(),
            )]))
            .build();
        OpenApiRouter::with_openapi(contract)
            .merge(AppRouter::router())
            .merge(SessionRouter::router())
    }
}
