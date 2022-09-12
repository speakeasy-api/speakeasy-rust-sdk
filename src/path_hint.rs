use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\{(.*?:*.*?)\}/|:(.+?)/|:(.*)|\*"#).unwrap());

pub fn normalize_path_hint(path_hint: String) -> String {
    RE.replace_all(&path_hint, |caps: &Captures| {
        // if its a wildcard return capture replacement early
        if &caps[0] == "*" {
            return Cow::Borrowed("{wildcard}");
        };

        // look through the captures and use the first non-empty capture
        get_first_capture(caps)
            .map(|matched| {
                if caps[0].ends_with("/") {
                    Cow::Owned(format!("{{{}}}/", matched))
                } else {
                    Cow::Owned(format!("{{{}}}", matched))
                }
            })
            .unwrap_or_else(|| Cow::Owned(caps[0].to_string()))
    })
    .into_owned()
}

fn get_first_capture<'a>(caps: &'a Captures) -> Option<&'a str> {
    for i in 1..caps.len() {
        if let Some(c) = caps.get(i) {
            return Some(c.as_str());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn works_on_simple_path() {
        let expected = "/hello/world".to_string();

        assert_eq!(expected, normalize_path_hint("/hello/world".to_string()));
    }

    #[test]
    fn normalizes_different_formats() {
        let normalized = "/user/{id}/account/{action}";

        assert_eq!(
            normalized,
            normalize_path_hint("/user/{id}/account/:action".to_string())
        );

        assert_eq!(
            normalized,
            normalize_path_hint("/user/:id/account/:action".to_string())
        );
    }

    #[test]
    fn normalizes_wildcard_path() {
        let normalized = "/user/{id}/account/{wildcard}";

        assert_eq!(
            normalized,
            normalize_path_hint("/user/:id/account/*".to_string())
        );
    }

    #[test]
    fn keeps_path_ending_in_slash() {
        let normalized = "/user/{id}/account/{action}/";

        assert_eq!(
            normalized,
            normalize_path_hint("/user/:id/account/{action}/".to_string())
        );
    }
}
