use chrono::Utc;
use har::{
    v1_2::{
        Cache, Content, Cookies as HarCookie, Creator, Entries as HarEntry, Headers as HarHeader,
        Log, PostData, QueryString, Request as HarRequest, Response as HarResponse, Timings,
    },
    Har,
};
use http::StatusCode;
use url::Url;

use crate::{
    generic_http::{BodyCapture, GenericRequest, GenericResponse, DROPPED_TEXT},
    masking::{
        body_mask::RequestMask,
        generic_mask::{GenericMask, QueryStringMask, RequestCookieMask, RequestHeaderMask},
    },
    masking::{
        body_mask::{BodyMask, ResponseMask},
        generic_mask::{ResponseCookieMask, ResponseHeaderMask},
    },
    Masking,
};

#[derive(Debug, Clone)]
pub struct HarBuilder {
    request: GenericRequest,
    response: GenericResponse,

    max_capture_size: Option<u64>,

    // helper to avoid cloning
    masked_full_url: Option<Url>,
    path_with_query: Option<String>,
}

impl HarBuilder {
    pub(crate) fn new(
        request: GenericRequest,
        response: GenericResponse,
        max_capture_size: Option<u64>,
    ) -> Self {
        Self {
            request,
            response,
            max_capture_size,
            masked_full_url: None,
            path_with_query: None,
        }
    }

    pub(crate) fn build(mut self, masking: &Masking) -> Har {
        self.masked_full_url = self.get_masked_full_url(masking);

        let path = self
            .masked_full_url
            .as_ref()
            .map(|u| u.path().to_string())
            .unwrap_or_else(|| self.request.path.clone());

        let path_with_query =
            if let Some(query) = self.masked_full_url.as_ref().and_then(|u| u.query()) {
                if query.is_empty() {
                    path
                } else {
                    format!("{}?{}", path, query)
                }
            } else {
                path
            };

        self.path_with_query = Some(path_with_query);

        Har {
            log: har::Spec::V1_2(Log {
                creator: Creator {
                    name: "speakeasy-rust-sdk".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    ..Default::default()
                },
                comment: Some(format!(
                    "request capture for {}",
                    &self.path_with_query.as_ref().expect("just set above")
                )),
                entries: vec![HarEntry {
                    started_date_time: self.request.start_time.to_rfc3339(),
                    time: Utc::now()
                        .signed_duration_since(self.request.start_time)
                        .num_milliseconds()
                        .abs() as f64,
                    request: self.build_request(masking),
                    response: self.build_response(masking),
                    cache: Cache::default(),
                    timings: Timings {
                        send: -1.0,
                        receive: -1.0,
                        wait: -1.0,
                        ..Default::default()
                    },
                    server_ip_address: Some(self.request.host.clone()),
                    connection: self.request.port.map(|p| p.to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
        }
    }

    fn build_request(&mut self, masking: &Masking) -> HarRequest {
        // drop body if controller was used to set a lower max capture size (request)
        if let (Some(max_capture_size), BodyCapture::Captured(body)) =
            (self.max_capture_size, &self.request.body)
        {
            if body.len() > max_capture_size as usize {
                self.request.body = BodyCapture::Dropped
            }
        }

        let body_size = if self.request.body == BodyCapture::Empty {
            -1
        } else {
            self.request
                .headers
                .get(http::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().unwrap().parse::<i64>().ok())
                .unwrap_or(-1)
        };

        HarRequest {
            method: self.request.method.clone(),
            url: self
                .path_with_query
                .as_ref()
                .expect("path_with_query should be set")
                .clone(),
            http_version: format!("{:?}", self.request.http_version),
            cookies: self.build_request_cookies(&masking.request_cookie_mask),
            headers: self.build_request_headers(&masking.request_header_mask),
            query_string: self.build_query_string(&masking.query_string_mask),
            headers_size: format!("{:?}", &self.request.headers).len() as i64,
            body_size,
            post_data: self.build_body_post_data(&masking.request_masks),
            comment: None,
        }
    }

    fn build_response(&mut self, masking: &Masking) -> HarResponse {
        // drop body if controller was used to set a lower max capture size (response)
        if let (Some(max_capture_size), BodyCapture::Captured(body)) =
            (self.max_capture_size, &self.response.body)
        {
            if body.len() > max_capture_size as usize {
                self.response.body = BodyCapture::Dropped
            }
        }

        HarResponse {
            status: self.response.status.as_u16() as i64,
            status_text: self.response.status.to_string(),
            http_version: format!("{:?}", &self.response.http_version),
            cookies: self.build_response_cookies(&masking.response_cookie_mask),
            headers: self.build_response_headers(&masking.response_header_mask),
            content: self.build_response_content(&masking.response_masks),
            redirect_url: self
                .response
                .headers
                .get("location")
                .and_then(|v| v.to_str().ok())
                .filter(|v| !v.is_empty())
                .map(ToString::to_string),
            headers_size: format!("{:?}", &self.response.headers).len() as i64,
            body_size: self.build_response_body_size(),
            comment: None,
        }
    }

    fn build_request_cookies(&self, masker: &GenericMask<RequestCookieMask>) -> Vec<HarCookie> {
        self.request
            .cookies
            .iter()
            .map(|cookie| HarCookie {
                name: cookie.name.clone(),
                value: masker.mask(&cookie.name, &cookie.value),
                ..Default::default()
            })
            .collect()
    }

    fn build_request_headers(&self, masker: &GenericMask<RequestHeaderMask>) -> Vec<HarHeader> {
        self.request
            .headers
            .iter()
            .map(|(name, value)| HarHeader {
                name: name.to_string(),
                value: masker.mask(name.as_str(), value.to_str().unwrap_or("")),
                comment: None,
            })
            .collect()
    }

    fn build_query_string(
        &self,
        query_string_mask: &GenericMask<QueryStringMask>,
    ) -> Vec<QueryString> {
        if let Some(url) = &self.request.full_url {
            url.query_pairs()
                .map(|(name, value)| QueryString {
                    name: name.to_string(),
                    value: query_string_mask.mask(&name, &value),
                    comment: None,
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn build_body_post_data(&self, masker: &BodyMask<RequestMask>) -> Option<PostData> {
        if self.request.body == BodyCapture::Empty {
            return None;
        }

        match self.request.body {
            BodyCapture::Empty => None,
            BodyCapture::Captured(ref text) => {
                let content_type = self
                    .request
                    .headers
                    .get(http::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("");

                let body_str = String::from_utf8_lossy(text);

                let body_string = if content_type == "application/json" {
                    masker.mask(&body_str)
                } else {
                    body_str.to_string()
                };

                Some(PostData {
                    mime_type: content_type.to_string(),
                    text: Some(body_string),
                    ..Default::default()
                })
            }
            BodyCapture::Dropped => {
                let content_type = self
                    .request
                    .headers
                    .get(http::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("application/octet-stream");

                Some(PostData {
                    mime_type: content_type.to_string(),
                    text: Some(DROPPED_TEXT.to_string()),
                    ..Default::default()
                })
            }
        }
    }

    fn build_response_cookies(&self, masker: &GenericMask<ResponseCookieMask>) -> Vec<HarCookie> {
        self.response
            .cookies
            .iter()
            .map(|cookie| HarCookie {
                name: cookie.name.clone(),
                value: masker.mask(&cookie.name, &cookie.value),
                ..Default::default()
            })
            .collect()
    }

    fn build_response_headers(&self, masker: &GenericMask<ResponseHeaderMask>) -> Vec<HarHeader> {
        self.response
            .headers
            .iter()
            .map(|(name, value)| HarHeader {
                name: name.to_string(),
                value: masker.mask(name.as_str(), value.to_str().unwrap_or("")),
                comment: None,
            })
            .collect()
    }

    fn build_response_content(&self, masker: &BodyMask<ResponseMask>) -> Content {
        let mime_type = self
            .response
            .headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        match self.request.body {
            BodyCapture::Empty => Content {
                size: -1,
                mime_type: Some(mime_type),
                ..Default::default()
            },
            BodyCapture::Dropped => Content {
                size: -1,
                text: Some(DROPPED_TEXT.to_string()),
                mime_type: Some(mime_type),
                ..Default::default()
            },
            BodyCapture::Captured(ref text) => {
                let body_str = String::from_utf8_lossy(text);

                let body_string = if &mime_type == "application/json" {
                    masker.mask(&body_str)
                } else {
                    body_str.to_string()
                };

                Content {
                    size: text.len() as i64,
                    text: Some(body_string),
                    mime_type: Some(mime_type),
                    ..Default::default()
                }
            }
        }
    }

    fn build_response_body_size(&self) -> i64 {
        if self.response.status == StatusCode::NOT_MODIFIED {
            0
        } else {
            self.response
                .headers
                .get(http::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().unwrap().parse::<i64>().ok())
                .unwrap_or(-1)
        }
    }

    fn get_masked_full_url(&self, masking: &Masking) -> Option<Url> {
        let mut url = self.request.full_url.as_ref()?.clone();

        let queries = url
            .query_pairs()
            .map(|(name, value)| {
                let masked_value = masking.query_string_mask.mask(&name, &value);
                (name.to_string(), masked_value)
            })
            .collect::<Vec<(String, String)>>();

        url.query_pairs_mut().clear().extend_pairs(queries);

        Some(url)
    }
}
