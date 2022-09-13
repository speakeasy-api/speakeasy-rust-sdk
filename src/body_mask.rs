use std::{borrow::Cow, collections::HashMap};

use regex::{Captures, Regex};
use std::fmt::Write as _;
use thiserror::Error;

use crate::util;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid string field name: {0}")]
    StringField(String),
    #[error("invalid number field name: {0}")]
    NumberField(String),
}

#[derive(Debug)]
pub struct BodyMask {
    string_masks: Option<Regex>,
    number_masks: Option<Regex>,
}

impl BodyMask {
    /// Create a new BodyMask struct using string_field_names and number_field_names
    /// The regex will be compiled and stored in the struct so it can be used reused, for repeated calls
    pub fn try_new(
        string_field_names: HashMap<String, String>,
        number_field_names: HashMap<String, i32>,
    ) -> Result<Self, Error> {
        let string_masks = if !string_field_names.is_empty() {
            let mut string_mask_regex = String::with_capacity(
                (string_field_names.len() * 32) + (string_field_names.len() * 24),
            );

            // build up single regex from string field regexes
            for (field_name, _replacement_value) in string_field_names {
                let _ = write!(
                    string_mask_regex,
                    r##"(?:("{}"): *)(".*?[^\\]")(?: *[, \n\r}}]?)|"##,
                    regex::escape(&field_name)
                );
            }

            // drop the last "|"
            string_mask_regex.pop();

            let string_masks = Regex::new(&string_mask_regex)
                .map_err(|_| Error::StringField(string_mask_regex))?;

            Some(string_masks)
        } else {
            None
        };

        let number_masks = if !number_field_names.is_empty() {
            // build number masks regex
            let mut number_mask_regex = String::with_capacity(
                (number_field_names.len() * 32) + (number_field_names.len() * 12),
            );

            for (field_name, _replacement_value) in number_field_names {
                let _ = write!(
                    number_mask_regex,
                    r##"(?:("{}"): *)(-?[0-9]+\.?[0-9]*)( *[, \n\r}}]?)|"##,
                    regex::escape(&field_name)
                );
            }

            // drop the last "|"
            number_mask_regex.pop();

            let number_masks = Regex::new(&number_mask_regex)
                .map_err(|_| Error::NumberField(number_mask_regex))?;

            Some(number_masks)
        } else {
            None
        };

        Ok(Self {
            string_masks,
            number_masks,
        })
    }

    /// Will use the regexes stored in the struct to mask the body
    pub fn mask(&self, body: String) -> String {
        // mask string fields
        let body = if let Some(string_mask_regex) = self.string_masks.as_ref() {
            string_mask_regex.replace_all(&body, |caps: &Captures| {
                if let Some(field) = util::get_first_capture(caps) {
                    format!(
                        r#"{}: "testmask"{}"#,
                        field,
                        caps[0].chars().last().unwrap()
                    )
                } else {
                    caps[0].to_string()
                }
            })
        } else {
            Cow::Owned(body)
        };

        // mask number fields
        let body = if let Some(number_mask_regex) = self.number_masks.as_ref() {
            number_mask_regex.replace_all(&body, |caps: &Captures| {
                if let Some(field) = util::get_first_capture(caps) {
                    format!(
                        r#"{}: -123456789{}"#,
                        field,
                        caps[0].chars().last().unwrap()
                    )
                } else {
                    caps[0].to_string()
                }
            })
        } else {
            body
        };

        body.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    struct Test {
        #[allow(dead_code)]
        name: &'static str,
        body: &'static str,
        expected: &'static str,
        string_masks: HashMap<String, String>,
        number_masks: HashMap<String, i32>,
    }

    #[test]
    fn run() {
        let tests: Vec<Test> = vec![
            Test {
                name: "successfully masks body with single string field",
                body: r#"{"test": "test"}"#,
                expected: r#"{"test": "testmask"}"#,
                string_masks: hashmap! {
                    "test".to_string() => "testmask".to_string(),
                },
                number_masks: hashmap! {},
            },
            Test {
                name: "successfully masks body with single int field",
                body: r#"{"test": 123}"#,
                expected: r#"{"test": -123456789}"#,
                string_masks: hashmap! {},
                number_masks: hashmap! {
                    "test".to_string() => -123456789,
                },
            },
            Test {
                name: "successfully masks body with single negative field",
                body: r#"{"test": -123}"#,
                expected: r#"{"test": -123456789}"#,
                string_masks: hashmap! {},
                number_masks: hashmap! {
                    "test".to_string() => -123456789,
                },
            },
            Test {
                name: "successfully masks body with single float field",
                body: r#"{"test": 123.123}"#,
                expected: r#"{"test": -123456789}"#,
                string_masks: hashmap! {},
                number_masks: hashmap! {
                    "test".to_string() => -123456789,
                },
            },
            Test {
                name: "successfully masks body with multiple masking fields",
                body: r#"{"test": "test", "another_test": "secret", "not_a_secret": "not a secret"}"#,
                expected: r#"{"test": "testmask", "another_test": "testmask", "not_a_secret": "not a secret"}"#,
                string_masks: hashmap! {
                    "test".to_string() => "testmask".to_string(),
                    "another_test".to_string() => "testmask".to_string(),
                },
                number_masks: hashmap! {},
            },
            Test {
                name: "successfully masks body with nested fields",
                body: r#"{"test": {"test": "test", "test1": 123}}"#,
                expected: r#"{"test": {"test": "testmask", "test1": -123456789}}"#,
                string_masks: hashmap! {
                    "test".to_string() => "testmask".to_string(),
                },
                number_masks: hashmap! {
                    "test1".to_string() => -123456789,
                },
            },
            Test {
                name: "successfully masks formatted body",
                body: r#"
                "test": {
                    "test": "test",
                    "test1": 123
                }"#,
                expected: r#"
                "test": {
                    "test": "testmask",
                    "test1": -123456789
                }"#,
                string_masks: hashmap! {
                    "test".to_string() => "testmask".to_string(),
                },
                number_masks: hashmap! {
                    "test1".to_string() => -123456789,
                },
            },
            Test {
                name: "successfully masks body with complex string field",
                body: r#"{"test": "\",{abc}: .\""}"#,
                expected: r#"{"test": "testmask"}"#,
                string_masks: hashmap! {
                    "test".to_string() => "testmask".to_string()
                },
                number_masks: hashmap! {},
            },
            Test {
                name: "successfully masks body with complex field key",
                body: r#"{"test\"hello\": ": "\",{abc}: .\""}"#,
                expected: r#"{"test\"hello\": ": "testmask"}"#,
                string_masks: hashmap! {
                    r#"test\"hello\": "#.to_string() => "testmask".to_string()
                },
                number_masks: hashmap! {},
            },
        ];

        for test in tests {
            assert_eq!(
                BodyMask::try_new(test.string_masks, test.number_masks)
                    .unwrap()
                    .mask(test.body.to_string()),
                test.expected,
            );
        }
    }
}
