use std::collections::HashMap;

use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid string field name: {0}")]
    StringField(String),
    #[error("invalid number field name: {0}")]
    NumberField(String),
}

pub struct BodyMask {
    string_masks: Regex,
    number_masks: Regex,
}

impl BodyMask {
    pub fn try_new(
        string_field_names: HashMap<String, String>,
        number_field_names: HashMap<String, i32>,
    ) -> Result<Self, Error> {
        let mut string_masks: Vec<(Regex, String)> = Vec::with_capacity(string_field_names.len());
        let string_mask_regex = String::with_capacity(
            (string_field_names.len() * 32) + (string_field_names.len() * 24),
        );

        // build up single regex from string field regexes
        for (field_name, replacement_value) in string_field_names {
            string_mask_regex.push_str(&format!(
                r##"("{}": *)(".*?[^\\]")( *[, \n\r}}]?)|"##,
                field_name
            ));
        }

        // drop the last "|"
        string_mask_regex.pop();

        let string_masks =
            Regex::new(&string_mask_regex).map_err(|_| Error::StringField(string_mask_regex))?;

        // build number masks regex
        let mut number_masks: Vec<(Regex, String)> = Vec::with_capacity(string_field_names.len());
        let number_mask_regex = String::with_capacity(
            (string_field_names.len() * 32) + (string_field_names.len() * 12),
        );

        for (field_name, replacement_value) in number_field_names {
            number_mask_regex.push_str(&format!(
                r##"("{}": *)(".*?[^\\]")( *[, \n\r}}]?)|"##,
                field_name
            ));
        }

        let number_masks =
            Regex::new(&number_mask_regex).map_err(|_| Error::NumberField(number_mask_regex))?;

        Ok(Self {
            string_masks,
            number_masks,
        })
    }

    pub fn mask(&self, body: String) -> String {}
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
        let tests: Vec<Test> = vec![Test {
            name: "successfully masks body with single string field",
            body: r#"{"test": "test"}"#,
            expected: r#"{"test": "testmask"}"#,
            string_masks: hashmap! {
                "test".to_string() => "testmask".to_string(),
            },
            number_masks: HashMap::new(),
        }];

        for test in tests {
            assert_eq!(
                test.expected,
                BodyMask::try_new_with_custom_masks(test.string_masks, test.number_masks)
                    .unwrap()
                    .mask(test.body.to_string())
            );
        }
    }
}
