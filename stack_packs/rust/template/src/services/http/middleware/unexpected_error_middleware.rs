use std::cell::{Cell, RefCell};
use std::future::Future;
use std::panic::{AssertUnwindSafe, PanicHookInfo};
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::Json;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::panic_exception::PanicException;
use crate::services::http::wire_models::error_response::ErrorResponse;

thread_local! {
    // How many requests this thread is polling inside the middleware right now, and the panic one of them raised.
    static CATCHING_DEPTH: Cell<usize> = const { Cell::new(0) };
    static CAUGHT_PANIC: RefCell<Option<PanicException>> = const { RefCell::new(None) };
}

// Logs, with its stack and the request it failed, any panic in a request (Rust's unexpected exception: the app's own
// exceptions are answered by ApplicationExceptionResponse), and answers it generically so nothing of the app's
// internals leaks. A panic in a response's body, after the response has started, is past it: the server ends that
// connection, and the panic hook logs it.
//
// A panic is seen with its stack only in the panic hook, which runs before the panic reaches this middleware. So that
// it is logged once, where it is caught, the hook hands a panic this middleware will catch to it (take_over_panic)
// instead of logging it.
pub struct UnexpectedErrorMiddleware;

impl UnexpectedErrorMiddleware {
    pub async fn handle(request: Request, next: Next) -> Response {
        let context: ContextualData = ContextualData::new()
            .with("method", request.method().as_str())
            .with("path", request.uri().path());

        match (CatchingPanics {
            request_future: Box::pin(next.run(request)),
        })
        .await
        {
            Ok(response) => response,
            Err(panic_exception) => {
                ApplicationLogger::error("A request failed unexpectedly", context, Some(&panic_exception));
                let error_response: ErrorResponse = ErrorResponse {
                    error: "The request could not be completed".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
        }
    }

    /// Called by the service's panic hook: keeps a panic this middleware is about to catch, with its stack, and says
    /// whether it did (the hook then logs nothing).
    pub fn take_over_panic(panic_hook_info: &PanicHookInfo<'_>) -> bool {
        if CATCHING_DEPTH.get() == 0 {
            return false;
        }
        CAUGHT_PANIC.set(Some(PanicException::from_panic_hook_info(panic_hook_info)));
        true
    }
}

// The request's future, polled so that a panic inside it is caught on the thread that polled it.
struct CatchingPanics {
    request_future: Pin<Box<dyn Future<Output = Response> + Send>>,
}

impl Future for CatchingPanics {
    type Output = Result<Response, PanicException>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        CATCHING_DEPTH.set(CATCHING_DEPTH.get() + 1);
        let polled: std::thread::Result<Poll<Response>> =
            std::panic::catch_unwind(AssertUnwindSafe(|| self.request_future.as_mut().poll(context)));
        CATCHING_DEPTH.set(CATCHING_DEPTH.get() - 1);

        match polled {
            Ok(Poll::Ready(response)) => Poll::Ready(Ok(response)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(payload) => Poll::Ready(Err(CAUGHT_PANIC
                .take()
                .unwrap_or_else(|| PanicException::from_payload(payload.as_ref())))),
        }
    }
}
