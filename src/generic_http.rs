use http::{version::Version, HeaderMap};

#[derive(Debug, Clone)]
pub(crate) struct GenericCookie {
    pub(crate) name: String,
    pub(crate) value: String,
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
}

/// A generic HTTP response, which can be converted to a HAR response
/// A generic HTTP response, can be created from a response from a web framework
#[derive(Debug, Clone)]
pub(crate) struct GenericResponse {}
