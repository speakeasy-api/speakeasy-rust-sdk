use std::{borrow::Cow, collections::HashMap};

use regex::{Captures, Regex};
use thiserror::Error;

use crate::util;

struct CaptureMatch<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid string field name: {0}")]
    StringField(String),
    #[error("invalid number field name: {0}")]
    NumberField(String),
}

pub struct BodyMask {
    string_masks: Option<Regex>,
    number_masks: Option<Regex>,
}

impl BodyMask {
    pub fn try_new(
        string_field_names: HashMap<String, String>,
        number_field_names: HashMap<String, i32>,
    ) -> Result<Self, Error> {
        let string_masks = if !string_field_names.is_empty() {
            let mut string_mask_regex = String::with_capacity(
                (string_field_names.len() * 32) + (string_field_names.len() * 24),
            );

            // // start string mask regex with a parent group
            // string_mask_regex.push('(');

            // build up single regex from string field regexes
            for (field_name, replacement_value) in string_field_names {
                string_mask_regex.push_str(&format!(
                    r##"(?:("{}"): *)(".*?[^\\]")(?: *[, \n\r}}]?)|"##,
                    field_name
                ));
            }

            // drop the last "|" and close off parent capture group
            string_mask_regex.pop();
            // string_mask_regex.push(')');

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

            // start number mask regex with a parent group
            number_mask_regex.push('(');

            for (field_name, replacement_value) in number_field_names {
                number_mask_regex.push_str(&format!(
                    r##"("{}": *)(".*?[^\\]")( *[, \n\r}}]?)|"##,
                    field_name
                ));
            }

            // drop the last "|" and close off parent capture group
            number_mask_regex.pop();
            number_mask_regex.push(')');

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

    pub fn mask(&self, body: String) -> String {
        let body = self
            .string_masks
            .as_ref()
            .unwrap()
            .replace_all(&body, |caps: &Captures| {
                if let Some(field) = util::get_first_capture(caps) {
                    format!(
                        r#"{}: "testmask"{}"#,
                        field,
                        caps[0].chars().last().unwrap()
                    )
                } else {
                    caps[0].to_string()
                }
            });

        body.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    struct Test {
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
                number_masks: HashMap::new(),
            },
            Test {
                name: "successfully masks body with multiple masking fields",
                body: r#"{"test": "test", "another_test": "secret", "not_a_secret": "not a secret"}"#,
                expected: r#"{"test": "testmask", "another_test": "testmask", "not_a_secret": "not a secret"}"#,
                string_masks: hashmap! {
                    "test".to_string() => "testmask".to_string(),
                    "another_test".to_string() => "testmask".to_string(),
                },
                number_masks: HashMap::new(),
            },
        ];

        for test in tests {
            assert_eq!(
                test.expected,
                BodyMask::try_new(test.string_masks, test.number_masks)
                    .unwrap()
                    .mask(test.body.to_string())
            );
        }
    }
}
