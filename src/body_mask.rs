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
    string_masks: Vec<(Regex, String)>,
    number_masks: Vec<(Regex, i32)>,
}

impl BodyMask {
    pub fn try_new_with_custom_masks(
        string_field_names: HashMap<String, String>,
        number_field_names: HashMap<String, i32>,
    ) -> Result<Self, Error> {
        let mut string_masks: Vec<(Regex, String)> = Vec::with_capacity(string_field_names.len());
        let mut number_masks: Vec<(Regex, i32)> = Vec::with_capacity(number_field_names.len());

        // build up string field regexes
        for (field_name, replacement_value) in string_field_names {
            let regex_string = format!(r##"("{}": *)(".*?[^\\]")( *[, \n\r}}]?)"##, field_name);

            let regex =
                Regex::new(&regex_string).map_err(|_| Error::StringField(field_name.clone()))?;

            string_masks.push((regex, replacement_value));
        }

        // build up number field regex's
        for (field_name, replacement_value) in number_field_names {
            let regex_string = format!(r##"("{}": *)(".*?[^\\]")( *[, \n\r}}]?)"##, field_name);

            let regex =
                Regex::new(&regex_string).map_err(|_| Error::NumberField(field_name.clone()))?;

            number_masks.push((regex, replacement_value));
        }

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
