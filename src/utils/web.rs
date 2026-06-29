//! Lightweight HTTP helpers for common API calls.
//!
//! Successful responses are returned as an [`HttpReply`], which preserves the
//! response status, headers, and body. The body is classified from the
//! `Content-Type` header into JSON, text, or raw bytes.

use std::sync::OnceLock;

pub use eyre::Result;
use eyre::bail;
pub use reqwest::{
    StatusCode, Url,
    header::{self, HeaderMap, HeaderValue},
};

use serde::Deserialize;
pub use serde_json::Value;

/// Response body decoded from the server response content type.
#[derive(Debug, Clone)]
pub enum ResponseBody {
    /// A JSON body decoded into [`serde_json::Value`].
    Json(Value),
    /// A textual body decoded as UTF-8 lossy when needed.
    Text(String),
    /// A raw binary body.
    Binary(Vec<u8>),
}

impl ResponseBody {
    /// Attempts to deserialize a JSON response body into `T`.
    ///
    /// This succeeds only when the body variant is [`ResponseBody::Json`].
    ///
    /// # Errors
    ///
    /// Returns an error when the body is not JSON or when deserializing the
    /// stored JSON value into `T` fails.
    pub fn json_try_into<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self {
            ResponseBody::Json(json) => serde_json::from_value(json.clone()).map_err(Into::into),
            _ => bail!("Response body is not JSON, cannot deserialize"),
        }
    }
}

/// Full HTTP reply returned by [`get_http`] and [`post_http`].
#[derive(Debug, Clone)]
pub struct HttpReply {
    /// Final HTTP status code returned by the server.
    pub status: StatusCode,
    /// Response headers returned by the server.
    pub headers: HeaderMap,
    /// Response body decoded from the `Content-Type` header.
    pub body: ResponseBody,
}

/// Default `User-Agent` value used when no custom value is provided.
///
/// The format is `<crate-name>/<crate-version>`.
pub const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(reqwest::Client::new)
}

fn prepare_headers(headers: Option<HeaderMap>, user_agent: Option<&str>) -> Result<HeaderMap> {
    let mut headers = headers.unwrap_or_default();

    if !headers.contains_key(header::USER_AGENT) {
        let value = HeaderValue::from_str(user_agent.unwrap_or(DEFAULT_USER_AGENT))?;
        headers.insert(header::USER_AGENT, value);
    }

    Ok(headers)
}

/// Decodes a successful response body from the response `Content-Type`.
///
/// Text responses become [`ResponseBody::Text`], JSON responses become
/// [`ResponseBody::Json`], and all other payloads are returned as
/// [`ResponseBody::Binary`].
///
/// # Errors
///
/// Returns an error only when a response marked as JSON cannot be parsed as
/// valid JSON.
fn decode_body(headers: &HeaderMap, body: &[u8]) -> Result<ResponseBody> {
    if let Some(mime_type) = headers.get(header::CONTENT_TYPE)
        && let Ok(mime_str) = mime_type.to_str()
    {
        if mime_str.starts_with("text/") {
            let text = String::from_utf8_lossy(body).to_string();
            return Ok(ResponseBody::Text(text));
        } else if mime_str.contains("json") {
            let json = serde_json::from_slice(body)?;
            return Ok(ResponseBody::Json(json));
        }
    }

    Ok(ResponseBody::Binary(body.to_vec()))
}

async fn decode_response(response: reqwest::Response) -> Result<HttpReply> {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes().await?;

    if !status.is_success() {
        let body = String::from_utf8_lossy(&body);
        return Err(eyre::eyre!("HTTP request failed: HTTP {} {}", status, body));
    }

    Ok(HttpReply {
        status,
        headers: headers.clone(),
        body: decode_body(&headers, &body)?,
    })
}

fn url_with_query(url: &str, query: Option<&[(&str, &str)]>) -> Result<Url> {
    Ok(match query {
        Some(query) => Url::parse_with_params(url, query)?,
        None => Url::parse(url)?,
    })
}

/// Sends an HTTP GET request.
///
/// The provided `headers` are forwarded to the request. Pass `None` to send no
/// custom headers. If no `User-Agent` header is present, the function inserts
/// one using `user_agent` or [`DEFAULT_USER_AGENT`] when `user_agent` is
/// `None`.
///
/// Query parameters are appended from `query`. Pass `None` to omit them.
///
/// Successful `2xx` responses are returned as [`HttpReply`]. The body is
/// classified from `Content-Type` into [`ResponseBody::Json`],
/// [`ResponseBody::Text`], or [`ResponseBody::Binary`].
///
/// # Errors
///
/// Returns an error if the `User-Agent` header value is invalid, the request
/// fails, or the server responds with a non-success status code.
pub async fn get_http(
    url: impl AsRef<str>,
    headers: Option<HeaderMap>,
    query: Option<&[(&str, &str)]>,
    user_agent: Option<&str>,
) -> Result<HttpReply> {
    let headers = prepare_headers(headers, user_agent)?;
    let url = url_with_query(url.as_ref(), query)?;
    let response = http_client().get(url).headers(headers).send().await?;

    decode_response(response).await
}

/// Sends an HTTP POST request with a JSON body.
///
/// The provided `body` is serialized as JSON and sent to the remote endpoint.
/// The provided `headers` are forwarded to the request. Pass `None` to send no
/// custom headers. If no `User-Agent` header is present, the function inserts
/// one using `user_agent` or [`DEFAULT_USER_AGENT`] when `user_agent` is
/// `None`.
///
/// Query parameters are appended from `query`. Pass `None` to omit them.
///
/// Successful `2xx` responses are returned as [`HttpReply`]. The body is
/// classified from `Content-Type` into [`ResponseBody::Json`],
/// [`ResponseBody::Text`], or [`ResponseBody::Binary`].
///
/// # Errors
///
/// Returns an error if the `User-Agent` header value is invalid, the request
/// fails, the request body cannot be serialized, or the server responds with a
/// non-success status code.
pub async fn post_http(
    url: impl AsRef<str>,
    headers: Option<HeaderMap>,
    query: Option<&[(&str, &str)]>,
    body: Option<Value>,
    user_agent: Option<&str>,
) -> Result<HttpReply> {
    let headers = prepare_headers(headers, user_agent)?;
    let url = url_with_query(url.as_ref(), query)?;
    let mut request = http_client().post(url).headers(headers);
    if let Some(body) = body {
        request = request
            .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(serde_json::to_vec(&body)?);
    }
    let response = request.send().await?;

    decode_response(response).await
}
