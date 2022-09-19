use crate::{
    generic_http::{BodyCapture, GenericCookie, GenericRequest, GenericResponse},
    middleware::speakeasy_header_name,
};
use actix3::dev::{ServiceRequest, ServiceResponse};
use actix_http::HttpMessage;
use chrono::Utc;

impl GenericRequest {
    pub fn new(request: &ServiceRequest, body: BodyCapture) -> Self {
        // NOTE IMPORTANT: have to get cookies before getting headers or there will be a BorrowMut
        // already borrowed error from actix
        let cookies = get_request_cookies(request);

        let scheme = request.connection_info().scheme().to_string();
        let path = request.uri().to_string();
        let host = request.connection_info().host().to_string();

        let url_string = format!("{}://{}{}", scheme, host, path);
        let full_url = url::Url::parse(&url_string).ok();

        let port = full_url.as_ref().and_then(|u| u.port());

        GenericRequest {
            start_time: Utc::now(),
            scheme,
            full_url,
            method: request.method().to_string(),
            host,
            path,
            http_version: request.version(),
            headers: get_request_headers(request),
            cookies,
            port,
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
        .filter(|(k, _)| k != &speakeasy_header_name())
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
