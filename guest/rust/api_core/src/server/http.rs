use std::{collections::HashMap, fmt};

use thiserror::Error;

use crate::{core::messages::HttpResponse, core::types::HttpMethod, global, internal::wit};

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
pub async fn get(
    url: impl AsRef<str>,
    headers: Option<HashMap<String, String>>,
) -> Result<Vec<u8>, HttpError> {
    let url = url.as_ref();
    let headers = headers.unwrap_or_default().into_iter().collect::<Vec<_>>();
    let response_id = wit::server_http::get(url, &headers);

    wait_for_response(response_id).await
}

/// Sends an HTTP POST request to the given URL, and returns the response body.
///
/// Any errors in sending or receiving will be returned as an [HttpError].
///
/// **NOTE**: This may be replaced with `wasi-http` support in the future,
/// which will allow the use of native Rust libraries like `reqwest`.
pub async fn post(
    url: impl AsRef<str>,
    headers: Option<HashMap<String, String>>,
    body: Option<&[u8]>,
) -> Result<Vec<u8>, HttpError> {
    let url = url.as_ref();
    let headers = headers.unwrap_or_default().into_iter().collect::<Vec<_>>();
    let response_id = wit::server_http::post(url, &headers, body);

    wait_for_response(response_id).await
}

async fn wait_for_response(response_id: u64) -> Result<Vec<u8>, HttpError> {
    let response = global::wait_for_runtime_message(move |message: &HttpResponse| {
        message.response_id == response_id
    })
    .await;

    match response.error {
        Some(error) => Err(HttpError(error)),
        None => Ok(response.body),
    }
}
