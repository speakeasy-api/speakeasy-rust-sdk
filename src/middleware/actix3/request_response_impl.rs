use std::cell::Ref;

use crate::generic_http::{BodyCapture, GenericCookie, GenericRequest};
use actix3::dev::ServiceRequest;
use actix_http::HttpMessage;

impl GenericRequest {
    pub fn new(request: &ServiceRequest, body: BodyCapture) -> Self {
        // NOTE IMPORTANT: have to get cookies before getting headers or there will be a BorrowMut
        // already borrowed error from actix
        let cookies = get_cookies(request);

        GenericRequest {
            method: request.method().to_string(),
            hostname: Some(request.connection_info().host().to_string()),
            url: request.uri().to_string(),
            http_version: request.version(),
            headers: get_headers(request),
            cookies,
            port: get_port(request.uri()),
            body,
        }
    }
}

fn get_headers(request: &ServiceRequest) -> http::HeaderMap {
    request
        .headers()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn get_cookies(cookies: &ServiceRequest) -> Vec<crate::generic_http::GenericCookie> {
    if let Ok(cookies) = &cookies.cookies() {
        cookies
            .iter()
            .map(|cookie| GenericCookie {
                name: cookie.name().to_string(),
                value: cookie.value().to_string(),
            })
            .collect()
    } else {
        vec![]
    }
}

fn get_port(uri: &http::Uri) -> Option<u16> {
    Some(uri.port()?.as_u16())
}
