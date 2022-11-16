use crate::generic_http::{BodyCapture, GenericCookie, GenericRequest, GenericResponse};
use actix_web::{
    cookie::Cookie,
    dev::{ServiceRequest, ServiceResponse},
};
use chrono::{DateTime, NaiveDateTime, Utc};

impl GenericRequest {
    pub fn new(
        request: &ServiceRequest,
        start_time: DateTime<Utc>,
        path_hint: Option<String>,
        body: BodyCapture,
    ) -> Self {
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
            start_time,
            path_hint,
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
        cookies.iter().cloned().map(Into::into).collect()
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
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn get_response_cookies<T>(response: &ServiceResponse<T>) -> Vec<GenericCookie> {
    let mut cookies: Vec<GenericCookie> = Vec::new();

    for cookie in response.response().cookies() {
        cookies.push(cookie.into())
    }

    cookies
}

impl<'a> From<Cookie<'a>> for GenericCookie {
    fn from(cookie: Cookie<'a>) -> Self {
        GenericCookie {
            name: cookie.name().to_string(),
            value: cookie.value().to_string(),
            path: cookie.path().map(ToString::to_string),
            domain: cookie.domain().map(ToString::to_string),
            expires: get_cookie_expiration(&cookie),
            http_only: cookie.http_only(),
            secure: cookie.secure(),
        }
    }
}

fn get_cookie_expiration(cookie: &Cookie) -> Option<DateTime<Utc>> {
    let expires_at = cookie.expires()?.datetime()?;

    let datetime = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(expires_at.unix_timestamp(), 0)?,
        Utc,
    );

    Some(datetime)
}
