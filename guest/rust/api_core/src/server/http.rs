use std::{cell::RefCell, fmt, rc::Rc, task::Poll};

use thiserror::Error;

use crate::{internal::wit, messages::HttpResponse, prelude::RuntimeMessage};

#[derive(Error, Debug, Clone)]
/// Errors that can occur when making an HTTP request.
pub struct HttpError(pub String);
impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP error: {}", self.0)
    }
}

/// Sends an HTTP GET request to the given URL, and returns the response body.
///
/// Any errors in sending or receiving will be returned as an [HttpError].
///
/// **NOTE**: This may be replaced with `wasi-http` support in the future,
/// which will allow the use of native Rust libraries like `reqwest`.
pub async fn get(url: &str) -> Result<Vec<u8>, HttpError> {
    let result = Rc::new(RefCell::new(None));

    wit::server_http::get(url);
    let mut listener = Some(HttpResponse::subscribe({
        let result = result.clone();
        let url = url.to_owned();
        move |response| {
            if response.url != url {
                return;
            }

            *result.borrow_mut() = Some(match response.error {
                Some(error) => Err(HttpError(error)),
                None => Ok(response.body),
            });
        }
    }));

    std::future::poll_fn(move |_cx| match &*result.borrow() {
        Some(r) => {
            let r = (*r).clone();
            if let Some(listener) = listener.take() {
                listener.stop();
            }
            Poll::Ready(r)
        }
        _ => Poll::Pending,
    })
    .await
}
