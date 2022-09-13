mod body_mask;
mod option;

pub type StringMaskingOption = option::StringMaskingOption;
pub type NumberMaskingOption = option::NumberMaskingOption;

use self::body_mask::BodyMask;

pub(crate) const DEFAULT_STRING_MASK: &str = "__masked__";
pub(crate) const DEFAULT_NUMBER_MASK: i32 = -12321;

#[derive(Debug, Clone, Default)]
pub struct Masking {
    response_masks: BodyMask,
    request_masks: BodyMask,
}

#[derive(Debug, Clone)]
pub struct Fields(Vec<String>);
impl From<Vec<String>> for Fields {
    fn from(fields: Vec<String>) -> Self {
        Self(fields)
    }
}

impl From<String> for Fields {
    fn from(field: String) -> Self {
        Self(vec![field])
    }
}

impl From<&str> for Fields {
    fn from(field: &str) -> Self {
        Self(vec![field.to_string()])
    }
}

impl From<&[&str]> for Fields {
    fn from(fields: &[&str]) -> Self {
        Self(fields.iter().map(ToString::to_string).collect())
    }
}

impl From<Fields> for Vec<String> {
    fn from(fields: Fields) -> Self {
        fields.0
    }
}

impl Masking {
    pub fn with_request_field_mask_string(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        todo!()
    }
}
