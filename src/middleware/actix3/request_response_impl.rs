use std::cell::Ref;

use crate::generic_http::{BodyCapture, GenericCookie, GenericRequest};
use actix3::dev::ServiceRequest;
use actix_http::{error::CookieParseError, http::Cookie, HttpMessage};

impl GenericRequest {
    pub fn new(request: &ServiceRequest, body: BodyCapture) -> Self {
        GenericRequest {
            method: request.method().to_string(),
            hostname: Some(request.connection_info().host().to_string()),
            url: request.uri().to_string(),
            http_version: request.version(),
            headers: request
                .headers()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            cookies: get_cookies(request.cookies()),
            port: get_port(request.uri()),
            body,
        }
    }
}

fn get_cookies(
    cookies: Result<Ref<Vec<Cookie>>, CookieParseError>,
) -> Vec<crate::generic_http::GenericCookie> {
    if let Ok(cookies) = cookies {
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
