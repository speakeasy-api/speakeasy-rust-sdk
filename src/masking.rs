mod body_mask;
mod fields;
mod option;

pub type StringMaskingOption = option::StringMaskingOption;
pub type NumberMaskingOption = option::NumberMaskingOption;

pub(crate) type Fields = fields::Fields;

use thiserror::Error;

use self::body_mask::BodyMask;

pub(crate) const DEFAULT_STRING_MASK: &str = "__masked__";
pub(crate) const DEFAULT_NUMBER_MASK: i32 = -12321;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid masking option {0}")]
    BodyMask(#[from] body_mask::Error),
}

#[derive(Debug, Clone, Default)]
pub struct Masking {
    response_masks: BodyMask,
    request_masks: BodyMask,
}

impl Masking {
    pub fn with_request_field_mask_string(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) -> Result<(), Error> {
        self.request_masks
            .set_string_field_masks(fields.into(), masking_option.into())?;

        Ok(())
    }

    pub fn with_request_field_mask_number(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<NumberMaskingOption>,
    ) -> Result<(), Error> {
        self.request_masks
            .set_number_field_masks(fields.into(), masking_option.into())?;

        Ok(())
    }

    pub fn with_response_field_mask_string(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) -> Result<(), Error> {
        self.response_masks
            .set_string_field_masks(fields.into(), masking_option.into())?;

        Ok(())
    }

    pub fn with_response_field_mask_number(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<NumberMaskingOption>,
    ) -> Result<(), Error> {
        self.response_masks
            .set_number_field_masks(fields.into(), masking_option.into())?;

        Ok(())
    }
}
