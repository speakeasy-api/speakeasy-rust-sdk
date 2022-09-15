use chrono::{DateTime, Utc};
use har::{
    v1_2::{
        Cache, Cookies as HarCookie, Creator, Entries as HarEntry, Headers as HarHeader, Log,
        QueryString, Request as HarRequest, Response as HarResponse, Timings,
    },
    Har,
};

use crate::{
    generic_http::{GenericRequest, GenericResponse},
    masking::generic_mask::{GenericMask, QueryStringMask, RequestCookieMask, RequestHeaderMask},
    Masking,
};

#[derive(Debug, Clone)]
pub struct HarBuilder {
    request: GenericRequest,
    response: GenericResponse,
    // TODO
    request_response_writer: Option<()>,
}

impl HarBuilder {
    pub(crate) fn new(
        request: impl Into<GenericRequest>,
        response: impl Into<GenericResponse>,
    ) -> Self {
        Self {
            request: request.into(),
            response: response.into(),
            request_response_writer: None,
        }
    }

    pub(crate) fn build(self, start_time: DateTime<Utc>, masking: &Masking) -> Har {
        Har {
            log: har::Spec::V1_2(Log {
                creator: Creator {
                    name: "speakeasy-rust-sdk".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    ..Default::default()
                },
                comment: Some(format!("request capture for {}", &self.request.full_url)),
                entries: vec![HarEntry {
                    started_date_time: start_time.to_rfc3339(),
                    time: Utc::now()
                        .signed_duration_since(start_time)
                        .num_milliseconds()
                        .abs() as f64,
                    request: self.build_request(masking),
                    response: todo!(),
                    cache: Cache::default(),
                    timings: Timings {
                        send: -1.0,
                        receive: -1.0,
                        wait: -1.0,
                        ..Default::default()
                    },
                    server_ip_address: self.request.hostname.clone(),
                    connection: self.request.port.map(|p| p.to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
        }
    }

    fn build_request(&self, masking: &Masking) -> HarRequest {
        HarRequest {
            method: self.request.method.clone(),
            url: self.request.full_url.clone(),
            http_version: format!("{:?}", self.request.http_version),
            cookies: self.build_request_cookies(&masking.request_cookie_mask),
            headers: self.build_request_headers(&masking.request_header_mask),
            query_string: self.build_query_string(&masking.query_string_mask),
            headers_size: self.build_request_headers_size(),
            body_size: todo!(),
            post_data: todo!(),
            ..Default::default()
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
        if let Ok(url) = url::Url::parse(&self.request.full_url) {
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

    fn build_request_headers_size(&self) -> i64 {
        format!("{:?}", &self.request.headers).len() as i64
    }
}
