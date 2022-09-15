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
    pub(crate) method: String,
    pub(crate) host: Option<String>,
    pub(crate) hostname: Option<String>,
    pub(crate) full_url: String,
    pub(crate) url: Option<String>,
    pub(crate) protocol: Option<String>,
    pub(crate) http_version: Version,
    pub(crate) headers: HeaderMap,
    pub(crate) cookies: Vec<GenericCookie>,
    pub(crate) port: Option<i32>,
    pub(crate) body: BodyCapture,
}

/// A generic HTTP response, which can be converted to a HAR response
/// A generic HTTP response, can be created from a response from a web framework
#[derive(Debug, Clone)]
pub(crate) struct GenericResponse {}
