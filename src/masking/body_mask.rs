use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use regex::{Captures, Regex};
use std::fmt::Write as _;
use thiserror::Error;

use crate::util;

use super::{fields::BodyMaskFieldsSearchMap, Fields, NumberMaskingOption, StringMaskingOption};

/// Errors for creating BodyMasks
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid string field name: {0}")]
    StringField(String),
    #[error("invalid number field name: {0}")]
    NumberField(String),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RequestMask;

#[derive(Debug, Clone, Default)]
pub(crate) struct ResponseMask;

/// BodyMasks holds information needed to perform masking on a request or response body
#[derive(Debug, Clone, Default)]
pub(crate) struct BodyMask<T> {
    phantom: PhantomData<T>,
    string_masks: Option<BodyMaskInner<StringMaskingOption>>,
    number_masks: Option<BodyMaskInner<NumberMaskingOption>>,
}

/// BodyMaskInner holds the regex, fields and options for masking
#[derive(Debug, Clone)]
pub(crate) struct BodyMaskInner<T> {
    regex: Regex,
    fields: BodyMaskFieldsSearchMap,
    mask_option: T,
}

impl From<BodyMaskInner<StringMaskingOption>> for HashMap<String, String> {
    fn from(mask: BodyMaskInner<StringMaskingOption>) -> Self {
        mask.fields
            .into_iter()
            .map(|(_field, (field_without_quotes, index))| {
                let value = mask
                    .mask_option
                    .get_mask_replacement(&field_without_quotes, index)
                    .to_string();

                (field_without_quotes, value)
            })
            .collect()
    }
}

impl From<BodyMaskInner<NumberMaskingOption>> for HashMap<String, String> {
    fn from(mask: BodyMaskInner<NumberMaskingOption>) -> Self {
        mask.fields
            .into_iter()
            .map(|(_field, (field_without_quotes, index))| {
                let value = mask
                    .mask_option
                    .get_mask_replacement(&field_without_quotes, index)
                    .to_string();

                (field_without_quotes, value)
            })
            .collect()
    }
}

// T = StringMaskingOption or NumberMaskingOption
impl<T> BodyMaskInner<T> {
    fn new(regex: Regex, fields: Fields, mask_option: T) -> Self {
        Self {
            regex,
            fields: fields.into(),
            mask_option,
        }
    }
}

impl<T: Default> BodyMask<T> {
    /// Checks if the masking is empty (not initialized)
    pub(crate) fn is_empty(&self) -> bool {
        self.string_masks.is_none() && self.number_masks.is_none()
    }

    /// Creates a BodyMask from a list of string fields to mask
    /// errors if there is a probably creating the Regex
    pub(crate) fn set_string_field_masks(
        &mut self,
        fields: Fields,
        masks_option: StringMaskingOption,
    ) -> Result<(), Error> {
        let string_masks = if !fields.is_empty() {
            let mut string_mask_regex =
                String::with_capacity((fields.len() * 32) + (fields.len() * 24));

            // build up single regex from string field regexes
            for field_name in fields.iter() {
                let _ = write!(
                    string_mask_regex,
                    r##"(?:("{}"): *)(".*?[^\\]")(?: *[, \n\r}}]?)|"##,
                    regex::escape(field_name)
                );
            }

            // drop the last "|"
            string_mask_regex.pop();

            let string_masks = Regex::new(&string_mask_regex)
                .map_err(|_| Error::StringField(string_mask_regex))?;

            Some(BodyMaskInner::new(string_masks, fields, masks_option))
        } else {
            None
        };

        self.string_masks = string_masks;

        Ok(())
    }

    /// Creates a BodyMask from a list of number fields to mask
    /// errors if there is a probably creating the Regex
    pub(crate) fn set_number_field_masks(
        &mut self,
        fields: Fields,
        masks_option: NumberMaskingOption,
    ) -> Result<(), Error> {
        let masks = if !fields.is_empty() {
            let mut mask_regex = String::with_capacity((fields.len() * 32) + (fields.len() * 24));

            // build up single regex from string field regexes
            for field_name in fields.iter() {
                let _ = write!(
                    mask_regex,
                    r##"(?:("{}"): *)(-?[0-9]+\.?[0-9]*)( *[, \n\r}}]?)|"##,
                    regex::escape(field_name)
                );
            }

            // drop the last "|"
            mask_regex.pop();

            let masks = Regex::new(&mask_regex).map_err(|_| Error::NumberField(mask_regex))?;

            Some(BodyMaskInner::new(masks, fields, masks_option))
        } else {
            None
        };

        self.number_masks = masks;

        Ok(())
    }

    /// Will use the regexes stored in the struct to mask the body
    pub fn mask(&self, body: &str) -> String {
        // mask string fields
        let body = if let Some(body_mask) = &self.string_masks {
            body_mask.regex.replace_all(body, |caps: &Captures| {
                if let Some(field) = util::get_first_capture(caps) {
                    let (field_without_quotes, index) =
                        body_mask.fields.get(field).unwrap_or_default();

                    let replacement_mask = body_mask
                        .mask_option
                        .get_mask_replacement(&field_without_quotes, index);

                    format!(
                        r#"{}:"{}"{}"#,
                        field,
                        replacement_mask,
                        caps[0].chars().last().unwrap()
                    )
                } else {
                    caps[0].to_string()
                }
            })
        } else {
            Cow::Borrowed(body)
        };

        // mask number fields
        let body = if let Some(body_mask) = &self.number_masks {
            body_mask.regex.replace_all(&body, |caps: &Captures| {
                if let Some(field) = util::get_first_capture(caps) {
                    let (field_without_quotes, index) =
                        body_mask.fields.get(field).unwrap_or_default();

                    let replacement_mask = body_mask
                        .mask_option
                        .get_mask_replacement(&field_without_quotes, index);

                    format!(
                        r#"{}:{}{}"#,
                        field,
                        replacement_mask,
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

    pub(crate) fn into_metadata(self) -> (HashMap<String, String>, HashMap<String, String>) {
        let string_masks = self.string_masks.map(|m| m.into()).unwrap_or_default();
        let number_masks = self.number_masks.map(|m| m.into()).unwrap_or_default();
        (string_masks, number_masks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    impl<T: Default> BodyMask<T> {
        /// Create a new BodyMask struct using string_field_names and number_field_names
        /// The regex will be compiled and stored in the struct so it can be used reused, for repeated calls
        pub(crate) fn try_new(
            string_field_names: HashMap<String, String>,
            number_field_names: HashMap<String, i32>,
        ) -> Result<Self, Error> {
            let mut body_mask = BodyMask::default();

            let mut string_fields = Vec::with_capacity(string_field_names.len());
            let mut string_masks = Vec::with_capacity(string_field_names.len());

            for (field_name, replacement_value) in &string_field_names {
                string_fields.push(field_name.clone());
                string_masks.push(replacement_value.clone());
            }

            // set string field masks
            body_mask.set_string_field_masks(
                string_fields.into(),
                StringMaskingOption::MultipleMasks(string_masks),
            )?;

            let mut number_fields = Vec::with_capacity(number_field_names.len());
            let mut number_masks = Vec::with_capacity(number_field_names.len());

            for (field_name, replacement_value) in &number_field_names {
                number_fields.push(field_name.clone());
                number_masks.push(*replacement_value);
            }

            // setup number field masks
            body_mask.set_number_field_masks(
                number_fields.into(),
                NumberMaskingOption::MultipleMasks(number_masks),
            )?;

            Ok(body_mask)
        }
    }

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
                BodyMask::<RequestMask>::try_new(test.string_masks, test.number_masks)
                    .unwrap()
                    .mask(test.body),
                test.expected,
            );
        }
    }
}
