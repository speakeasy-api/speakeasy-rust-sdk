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

        // build up number field regexes
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
}
