//! Extract out the host from the request.
//! Modified from: https://docs.rs/axum/0.5.17/src/axum/extract/host.rs.html

use http::{header::FORWARDED, HeaderMap, Request};
const X_FORWARDED_HOST_HEADER_KEY: &str = "X-Forwarded-Host";

#[derive(Debug, Clone)]
pub(crate) struct Host(pub String);

impl Host {
    pub(crate) fn from_request<T>(req: &Request<T>) -> Option<Host> {
        if let Some(host) = parse_forwarded(req.headers()) {
            return Some(Host(host.to_owned()));
        }

        if let Some(host) = req
            .headers()
            .get(X_FORWARDED_HOST_HEADER_KEY)
            .and_then(|host| host.to_str().ok())
        {
            return Some(Host(host.to_owned()));
        }

        if let Some(host) = req
            .headers()
            .get(http::header::HOST)
            .and_then(|host| host.to_str().ok())
        {
            return Some(Host(host.to_owned()));
        }

        if let Some(host) = req.uri().host() {
            return Some(Host(host.to_owned()));
        }

        None
    }

    pub(crate) fn take_string(self) -> String {
        self.0
    }
}

#[allow(warnings)]
fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
    // if there are multiple `Forwarded` `HeaderMap::get` will return the first one
    let forwarded_values = headers.get(FORWARDED)?.to_str().ok()?;

    // get the first set of values
    let first_value = forwarded_values.split(',').nth(0)?;

    // find the value of the `host` field
    first_value.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("host")
            .then(|| value.trim().trim_matches('"'))
    })
}
