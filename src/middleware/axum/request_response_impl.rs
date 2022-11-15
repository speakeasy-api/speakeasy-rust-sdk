use crate::{
    generic_http::{BodyCapture, GenericCookie, GenericRequest, GenericResponse},
    middleware::host_extract::Host,
};
use axum::{body::Body, http::Request};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{DateTime, NaiveDateTime, Utc};
use http::{header::COOKIE, Response};

impl GenericRequest {
    pub fn new(
        request: &Request<Body>,
        start_time: DateTime<Utc>,
        path_hint: Option<String>,
        body: BodyCapture,
    ) -> Self {
        // NOTE IMPORTANT: have to get cookies before getting headers or there will be a BorrowMut
        let cookies = get_request_cookies(request);

        let scheme = request.uri().scheme_str().unwrap_or("http").to_string();
        let path = request.uri().path().to_string();

        let host = if let Some(host) = Host::from_request(request) {
            host.take_string()
        } else {
            log::debug!("unable to extract host, falling back to localhost");
            "localhost".to_string()
        };

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

fn get_request_headers(request: &Request<Body>) -> http::HeaderMap {
    request
        .headers()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn get_request_cookies(request: &Request<Body>) -> Vec<GenericCookie> {
    if let Some(cookies) = request.extensions().get::<CookieJar>() {
        cookies.iter().cloned().map(Into::into).collect()
    } else {
        vec![]
    }
}

impl GenericResponse {
    pub(crate) fn new<T>(response: &Response<T>) -> Self {
        let status = response.status();
        let http_version = response.version();
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

fn get_response_headers<T>(response: &Response<T>) -> http::HeaderMap {
    response
        .headers()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn get_response_cookies<T>(response: &Response<T>) -> Vec<GenericCookie> {
    response
        .headers()
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
        .map(Into::into)
        .collect()
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
