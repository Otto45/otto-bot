use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use utoipa::openapi::{Info, OpenApi, OpenApiBuilder};
use utoipa_axum::router::OpenApiRouter;

use crate::example_app_facade::ExampleAppFacade;
use crate::services::http::middleware::unexpected_error_middleware::UnexpectedErrorMiddleware;
use crate::services::http::routers::customer_router::CustomerRouter;
use crate::services::http::routers::user_profile_router::UserProfileRouter;

// A service runs and listens. It sits outside the domain container and holds no application logic: it works the way
// its framework does, hands the Domain Facade only domain models, maps the domain models it gets back to response
// models, and translates the facade's exceptions into HTTP errors (ApplicationExceptionResponse).
pub struct HttpService;

impl HttpService {
    /// Creates the Domain Facade, serves requests with it until `shutdown` completes, then closes it. The facade is
    /// created once and lives as long as the server: every request gets this one instance from the router's state,
    /// and it is closed (with everything the locator cached) when the server has stopped, whether serving failed or
    /// not.
    pub async fn serve(
        listener: TcpListener,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // e.g. invalid configuration, which fails while the facade is created
        let example_app_facade: Arc<ExampleAppFacade> = Arc::new(ExampleAppFacade::new()?);

        let served: std::io::Result<()> =
            axum::serve(listener, HttpService::get_application(example_app_facade.clone()))
                .with_graceful_shutdown(shutdown)
                .await;

        example_app_facade.close().await;
        Ok(served?)
    }

    pub fn get_application(example_app_facade: Arc<ExampleAppFacade>) -> Router {
        let (router, _contract): (Router<Arc<ExampleAppFacade>>, OpenApi) = HttpService::build().split_for_parts();
        router
            .layer(axum::middleware::from_fn(UnexpectedErrorMiddleware::handle))
            .with_state(example_app_facade)
    }

    /// The service's contract, built from the routers. Building it creates no facade, so it needs no configuration.
    pub fn get_contract() -> OpenApi {
        HttpService::build().into_openapi()
    }

    fn build() -> OpenApiRouter<Arc<ExampleAppFacade>> {
        OpenApiRouter::with_openapi(OpenApiBuilder::new().info(Info::new("Example App", "0.1.0")).build())
            .merge(CustomerRouter::router())
            .merge(UserProfileRouter::router())
    }
}
