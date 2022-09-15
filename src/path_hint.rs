use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use crate::util;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\{(.*?:*.*?)\}/|:(.+?)/|:(.*)|\*"#).unwrap());

/// Normalize the path hint given from different web frameworks to a the OpenAPI spec
pub fn normalize_path_hint(path_hint: String) -> String {
    RE.replace_all(&path_hint, |caps: &Captures| {
        // if its a wildcard return capture replacement early
        if &caps[0] == "*" {
            return Cow::Borrowed("{wildcard}");
        };

        // look through the captures and use the first non-empty capture
        if let Some(matched) = util::get_first_capture(caps) {
            if caps[0].ends_with('/') {
                Cow::Owned(format!("{{{}}}/", matched))
            } else {
                Cow::Owned(format!("{{{}}}", matched))
            }
        } else {
            Cow::Owned(caps[0].to_string())
        }
    })
    .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    struct Test {
        #[allow(dead_code)]
        name: &'static str,
        path_hint: &'static str,
        expected: &'static str,
    }

    #[test]
    fn run() {
        let tests = vec![
            Test {
                name: "simple path",
                path_hint: "/hello/world",
                expected: "/hello/world",
            },
            Test {
                name: "simple path with wildcard",
                path_hint: "/hello/*",
                expected: "/hello/{wildcard}",
            },
            Test {
                name: "path with mix formats",
                path_hint: "/user/{id}/account/:action",
                expected: "/user/{id}/account/{action}",
            },
            Test {
                name: "path with multiple of the same format",
                path_hint: "/user/:id/account/:action",
                expected: "/user/{id}/account/{action}",
            },
            Test {
                name: "path with multiple of the same format (:)",
                path_hint: "/user/{id}/account/{action}",
                expected: "/user/{id}/account/{action}",
            },
            Test {
                name: "does not normalize unknown format",
                path_hint: "/user/<id>/account/<action>",
                expected: "/user/<id>/account/<action>",
            },
            Test {
                name: "keeps trailing slash",
                path_hint: "/user/{id}/account/{action}/",
                expected: "/user/{id}/account/{action}/",
            },
        ];

        for test in tests {
            assert_eq!(
                normalize_path_hint(test.path_hint.to_string()),
                test.expected
            );
        }
    }
}
