use std::fmt;

use thiserror::Error;

use crate::{core::messages::HttpResponse, global, internal::wit};

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
pub async fn get(url: impl AsRef<str>) -> Result<Vec<u8>, HttpError> {
    let url = url.as_ref();
    wit::server_http::get(url);

    let response = global::wait_for_runtime_message({
        let url = url.to_owned();
        move |message: &HttpResponse| message.url == url
    })
    .await;

    match response.error {
        Some(error) => Err(HttpError(error)),
        None => Ok(response.body),
    }
}
