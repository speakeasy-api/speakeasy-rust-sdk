use chrono::{DateTime, SecondsFormat, Utc};
use har::v1_2::Cookies as HarCookie;
use http::{version::Version, HeaderMap};

use crate::masking::generic_mask::GenericMask;

// len => 11
pub(crate) const DROPPED_TEXT: &str = "--dropped--";

#[derive(Debug, Clone)]
pub(crate) struct GenericCookie {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) path: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) expires: Option<DateTime<Utc>>,
    pub(crate) http_only: Option<bool>,
    pub(crate) secure: Option<bool>,
}

impl GenericCookie {
    pub(crate) fn into_har_cookie<T>(self, masker: &GenericMask<T>) -> HarCookie {
        HarCookie {
            name: self.name.clone(),
            value: masker.mask(&self.name, &self.value),
            path: self.path.clone(),
            domain: self.domain.clone(),
            expires: self
                .expires
                .as_ref()
                .map(|exp| exp.to_rfc3339_opts(SecondsFormat::Millis, true)),
            http_only: self.http_only,
            secure: self.secure,
            comment: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BodyCapture {
    // drop if > max, max = 1 * 1024 * 1024;
    Empty,
    Dropped,
    Captured(bytes::Bytes),
}

/// A generic HTTP request, which can be converted to a HAR request
/// A generic HTTP request, can be created from a request from a web framework
#[derive(Debug, Clone)]
pub struct GenericRequest {
    pub(crate) start_time: DateTime<Utc>,
    pub(crate) path_hint: Option<String>,
    pub(crate) full_url: Option<url::Url>,
    pub(crate) method: String,
    pub(crate) host: String,
    pub(crate) path: String,
    pub(crate) http_version: Version,
    pub(crate) headers: HeaderMap,
    pub(crate) cookies: Vec<GenericCookie>,
    pub(crate) port: Option<u16>,
    pub(crate) body: BodyCapture,
}

/// A generic HTTP response, which can be converted to a HAR response
/// A generic HTTP response, can be created from a response from a web framework
#[derive(Debug, Clone)]
pub struct GenericResponse {
    pub(crate) status: http::StatusCode,
    pub(crate) http_version: Version,
    pub(crate) headers: HeaderMap,
    pub(crate) cookies: Vec<GenericCookie>,
    pub(crate) body: BodyCapture,
}
