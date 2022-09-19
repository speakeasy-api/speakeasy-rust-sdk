use crate::generic_http::{BodyCapture, GenericCookie, GenericRequest, GenericResponse};
use actix3::dev::{ServiceRequest, ServiceResponse};
use actix_http::HttpMessage;
use chrono::Utc;

impl GenericRequest {
    pub fn new(request: &ServiceRequest, body: BodyCapture) -> Self {
        // NOTE IMPORTANT: have to get cookies before getting headers or there will be a BorrowMut
        // already borrowed error from actix
        let cookies = get_request_cookies(request);

        GenericRequest {
            start_time: Utc::now(),
            method: request.method().to_string(),
            hostname: Some(request.connection_info().host().to_string()),
            url: request.uri().to_string(),
            http_version: request.version(),
            headers: get_request_headers(request),
            cookies,
            port: get_port(request.uri()),
            body,
        }
    }
}

fn get_request_headers(request: &ServiceRequest) -> http::HeaderMap {
    request
        .headers()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn get_request_cookies(cookies: &ServiceRequest) -> Vec<GenericCookie> {
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

impl GenericResponse {
    pub(crate) fn new<T>(response: &ServiceResponse<T>) -> Self {
        let status = response.status();
        let http_version = response.request().version();
        let cookies = get_response_cookies(response);

        Self {
            status,
            http_version,
            headers: get_response_headers(response),
            cookies,
            body: BodyCapture::Empty,
        }
    }
}

fn get_response_headers<T>(response: &ServiceResponse<T>) -> http::HeaderMap {
    response
        .headers()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn get_response_cookies<T>(response: &ServiceResponse<T>) -> Vec<GenericCookie> {
    let mut cookies = Vec::new();

    for cookie in response.response().cookies() {
        cookies.push(GenericCookie {
            name: cookie.name().to_string(),
            value: cookie.value().to_string(),
        })
    }

    cookies
}
