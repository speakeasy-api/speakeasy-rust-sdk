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
    /**
    with_request_field_mask_string will mask the specified request body fields with an optional mask.
    Supports string fields only. Matches using regex.
    If no mask is provided, the value will be masked with the default mask.
    If a single mask is provided, it will be used for all fields.
    If the number of masks provided is equal to the number of fields, masks will be used in order.
    Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    (defaults to "__masked__").

    # Examples
    ```rust
        use std::collections::HashMap;
        use speakeasy_rust_sdk::{Masking, StringMaskingOption};

        // Mask a single field with the default mask
        let mut masking = Masking::default();
        masking.with_request_field_mask_string("password", StringMaskingOption::default());

        // Mask a single field with the default mask just using None
        let mut masking = Masking::default();
        masking.with_request_field_mask_string("password", None);

        // Mask a single field with a custom mask
        let mut masking = Masking::default();
        masking.with_request_field_mask_string("password", "************");

        // Mask multiple fields with a multiple masks
        let mut masking = Masking::default();
        masking.with_request_field_mask_string(
            ["authorization", "password", "more_secrets"].as_slice(),
            ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
        );

        // Mask multiple fields with a multiple masks
        let mut masking = Masking::default();
        masking.with_request_field_mask_string(
            vec!["authorization", "password", "more_secrets"],
            vec!["__my_mask__", "*****"],
        );

        // Mask multiple fields with multiple associated masks
        let mut customs_masks = HashMap::new();
        customs_masks.insert("authorization", "*****");
        customs_masks.insert("password", "hunter2");
        customs_masks.insert("more_secrets", "__my_mask__");

        let mut masking = Masking::default();
        masking.with_request_field_mask_string(
            vec!["authorization", "password", "more_secrets"],
            customs_masks
        );
    ```
    */
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
