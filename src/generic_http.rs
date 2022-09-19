use chrono::{DateTime, Utc};
use http::{version::Version, HeaderMap};

pub(crate) const DROPPED_TEXT: &str = "--dropped--";

#[derive(Debug, Clone)]
pub(crate) struct GenericCookie {
    pub(crate) name: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BodyCapture {
    // drop if > max, max = 1 * 1024 * 1024;
    Empty,
    Dropped,
    Captured(bytes::Bytes),
}

/// A generic HTTP request, which can be converted to a HAR request
/// A generic HTTP request, can be created from a request from a web framework
#[derive(Debug, Clone)]
pub(crate) struct GenericRequest {
    pub(crate) start_time: DateTime<Utc>,
    pub(crate) method: String,
    pub(crate) hostname: Option<String>,
    pub(crate) url: String,
    pub(crate) http_version: Version,
    pub(crate) headers: HeaderMap,
    pub(crate) cookies: Vec<GenericCookie>,
    pub(crate) port: Option<u16>,
    pub(crate) body: BodyCapture,
}

/// A generic HTTP response, which can be converted to a HAR response
/// A generic HTTP response, can be created from a response from a web framework
#[derive(Debug, Clone)]
pub(crate) struct GenericResponse {
    pub(crate) status: http::StatusCode,
    pub(crate) http_version: Version,
    pub(crate) headers: HeaderMap,
    pub(crate) cookies: Vec<GenericCookie>,
    pub(crate) body: BodyCapture,
}
