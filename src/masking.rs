mod fields;
mod option;

pub(crate) mod body_mask;
pub(crate) mod generic_mask;

pub type StringMaskingOption = option::StringMaskingOption;
pub type NumberMaskingOption = option::NumberMaskingOption;

pub(crate) type Fields = fields::Fields;

use std::collections::HashMap;

use log::error;
use speakeasy_protos::ingest::ingest_request::MaskingMetadata;

use self::{
    body_mask::{BodyMask, RequestMask, ResponseMask},
    generic_mask::{
        GenericMask, QueryStringMask, RequestCookieMask, RequestHeaderMask, ResponseCookieMask,
        ResponseHeaderMask,
    },
};

pub(crate) const DEFAULT_STRING_MASK: &str = "__masked__";
pub(crate) const DEFAULT_NUMBER_MASK: i32 = -12321;

/// All masking options, see for functions for more details on setting them
#[derive(Debug, Clone, Default)]
pub struct Masking {
    pub(crate) query_string_mask: GenericMask<QueryStringMask>,
    pub(crate) request_header_mask: GenericMask<RequestHeaderMask>,
    pub(crate) response_header_mask: GenericMask<ResponseHeaderMask>,
    pub(crate) request_cookie_mask: GenericMask<RequestCookieMask>,
    pub(crate) response_cookie_mask: GenericMask<ResponseCookieMask>,
    pub(crate) request_masks: BodyMask<RequestMask>,
    pub(crate) response_masks: BodyMask<ResponseMask>,
}

impl Masking {
    /// with_query_string_mask will mask the specified query strings with an optional mask string.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all query strings.
    /// If the number of masks provided is equal to the number of query strings, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_query_string_mask("password", StringMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_query_string_mask("password", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_query_string_mask("password", "************");
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_query_string_mask(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_query_string_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", "*****");
    /// customs_masks.insert("password", "hunter2");
    /// customs_masks.insert("more_secrets", "__my_mask__");
    ///
    /// let mut masking = Masking::default();
    /// masking.with_query_string_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks,
    /// );
    pub fn with_query_string_mask(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        self.query_string_mask = GenericMask::new(fields.into(), masking_option.into());
    }

    /// with_request_header_mask will mask the specified request headers with an optional mask string.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all headers.
    /// If the number of masks provided is equal to the number of headers, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_request_header_mask("password", StringMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_request_header_mask("password", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_request_header_mask("password", "************");
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_header_mask(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_header_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", "*****");
    /// customs_masks.insert("password", "hunter2");
    /// customs_masks.insert("more_secrets", "__my_mask__");
    ///
    /// let mut masking = Masking::default();
    /// masking.with_request_header_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks,
    /// );
    pub fn with_request_header_mask(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        self.request_header_mask = GenericMask::new(fields.into(), masking_option.into());
    }

    /// with_response_cookie_mask will mask the specified response cookies with an optional mask string.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all cookies.
    /// If the number of masks provided is equal to the number of cookies, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_response_cookie_mask("password", StringMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_response_cookie_mask("password", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_response_cookie_mask("password", "************");
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_response_cookie_mask(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_response_cookie_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", "*****");
    /// customs_masks.insert("password", "hunter2");
    /// customs_masks.insert("more_secrets", "__my_mask__");
    ///
    /// let mut masking = Masking::default();
    /// masking.with_response_cookie_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks,
    /// );
    pub fn with_response_cookie_mask(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        self.response_cookie_mask = GenericMask::new(fields.into(), masking_option.into());
    }

    /// with_response_header_mask will mask the specified response headers with an optional mask string.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all headers.
    /// If the number of masks provided is equal to the number of headers, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_response_header_mask("password", StringMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_response_header_mask("password", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_response_header_mask("password", "************");
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_response_header_mask(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_response_header_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", "*****");
    /// customs_masks.insert("password", "hunter2");
    /// customs_masks.insert("more_secrets", "__my_mask__");
    ///
    /// let mut masking = Masking::default();
    /// masking.with_response_header_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks,
    /// );
    pub fn with_response_header_mask(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        self.response_header_mask = GenericMask::new(fields.into(), masking_option.into());
    }

    /// with_request_cookie_mask will mask the specified request cookies with an optional mask string.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all cookies.
    /// If the number of masks provided is equal to the number of cookies, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_request_cookie_mask("password", StringMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_request_cookie_mask("password", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_request_cookie_mask("password", "************");
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_cookie_mask(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_cookie_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", "*****");
    /// customs_masks.insert("password", "hunter2");
    /// customs_masks.insert("more_secrets", "__my_mask__");
    ///
    /// let mut masking = Masking::default();
    /// masking.with_request_cookie_mask(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks,
    /// );
    pub fn with_request_cookie_mask(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        self.request_cookie_mask = GenericMask::new(fields.into(), masking_option.into());
    }

    /// with_request_field_mask_string will mask the specified request body fields with an optional mask.
    /// Supports string fields only. Matches using regex.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all fields.
    /// If the number of masks provided is equal to the number of fields, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_string("password", StringMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_string("password", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_string("password", "************");
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_string(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_string(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", "*****");
    /// customs_masks.insert("password", "hunter2");
    /// customs_masks.insert("more_secrets", "__my_mask__");
    ///
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_string(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks
    /// );
    /// ```
    pub fn with_request_field_mask_string(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        if let Err(err) = self
            .request_masks
            .set_string_field_masks(fields.into(), masking_option.into())
        {
            error!(
                "[SpeakeasySDK Internal error] - invalid request field mask string: {}",
                err
            );
        }
    }

    /// with_request_field_mask_number will mask the specified request body fields with an optional mask. Supports number fields only. Matches using regex.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all fields.
    /// If the number of masks provided is equal to the number of fields, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "-12321").
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, NumberMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_number("credit_card", NumberMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_number("SIN", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_number("SSN", 0);
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_number(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     [-1, -2, -3].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_number(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec![-111111, -222222],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", -1);
    /// customs_masks.insert("password", -11);
    /// customs_masks.insert("more_secrets", -111);
    ///
    /// let mut masking = Masking::default();
    /// masking.with_request_field_mask_number(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks
    /// );
    /// ```
    pub fn with_request_field_mask_number(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<NumberMaskingOption>,
    ) {
        if let Err(err) = self
            .request_masks
            .set_number_field_masks(fields.into(), masking_option.into())
        {
            error!(
                "[SpeakeasySDK Internal error] - invalid request field mask string: {}",
                err
            );
        }
    }

    /// with_response_field_string will mask the specified response body with an optional mask. Supports string only. Matches using regex.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all fields.
    /// If the number of masks provided is equal to the number of fields, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "__masked__").
    ///
    ///  # Examples
    ///  ```rust
    ///  use std::collections::HashMap;
    ///  use speakeasy_rust_sdk::{Masking, StringMaskingOption};
    ///
    ///  // Mask a single field with the default mask
    ///  let mut masking = Masking::default();
    ///  masking.with_response_field_mask_string("password", StringMaskingOption::default());
    ///
    ///  // Mask a single field with the default mask just using None
    ///  let mut masking = Masking::default();
    ///  masking.with_response_field_mask_string("password", None);
    ///
    ///  // Mask a single field with a custom mask
    ///  let mut masking = Masking::default();
    ///  masking.with_response_field_mask_string("password", "************");
    ///
    ///  // Mask multiple fields with a multiple masks
    ///  let mut masking = Masking::default();
    ///  masking.with_response_field_mask_string(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     ["__masked__", "*****", "no_secrets_for_you"].as_slice(),
    ///  );
    ///
    ///  // Mask multiple fields with a multiple masks
    ///  let mut masking = Masking::default();
    ///  masking.with_response_field_mask_string(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec!["__my_mask__", "*****"],
    ///  );
    ///
    ///  // Mask multiple fields with multiple associated masks
    ///  let mut customs_masks = HashMap::new();
    ///  customs_masks.insert("authorization", "*****");
    ///  customs_masks.insert("password", "hunter2");
    ///  customs_masks.insert("more_secrets", "__my_mask__");
    ///
    ///  let mut masking = Masking::default();
    ///  masking.with_response_field_mask_string(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks
    ///  );
    ///  ```
    pub fn with_response_field_mask_string(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<StringMaskingOption>,
    ) {
        if let Err(err) = self
            .response_masks
            .set_string_field_masks(fields.into(), masking_option.into())
        {
            error!(
                "[SpeakeasySDK Internal error] - invalid response field mask string: {}",
                err
            );
        }
    }

    /// with_response_field_mask_number will mask the specified response body with an optional mask. Supports number fields only. Matches using regex.
    /// If no mask is provided, the value will be masked with the default mask.
    /// If a single mask is provided, it will be used for all fields.
    /// If the number of masks provided is equal to the number of fields, masks will be used in order.
    /// Otherwise, the masks will be used in order until it they are exhausted. If the masks are exhausted, the default mask will be used.
    /// (defaults to "-12321").
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use speakeasy_rust_sdk::{Masking, NumberMaskingOption};
    ///
    /// // Mask a single field with the default mask
    /// let mut masking = Masking::default();
    /// masking.with_response_field_mask_number("credit_card", NumberMaskingOption::default());
    ///
    /// // Mask a single field with the default mask just using None
    /// let mut masking = Masking::default();
    /// masking.with_response_field_mask_number("SIN", None);
    ///
    /// // Mask a single field with a custom mask
    /// let mut masking = Masking::default();
    /// masking.with_response_field_mask_number("SSN", 0);
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_response_field_mask_number(
    ///     ["authorization", "password", "more_secrets"].as_slice(),
    ///     [-1, -2, -3].as_slice(),
    /// );
    ///
    /// // Mask multiple fields with a multiple masks
    /// let mut masking = Masking::default();
    /// masking.with_response_field_mask_number(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     vec![-111111, -222222],
    /// );
    ///
    /// // Mask multiple fields with multiple associated masks
    /// let mut customs_masks = HashMap::new();
    /// customs_masks.insert("authorization", -1);
    /// customs_masks.insert("password", -11);
    /// customs_masks.insert("more_secrets", -111);
    ///
    /// let mut masking = Masking::default();
    /// masking.with_response_field_mask_number(
    ///     vec!["authorization", "password", "more_secrets"],
    ///     customs_masks
    /// );
    /// ```
    pub fn with_response_field_mask_number(
        &mut self,
        fields: impl Into<Fields>,
        masking_option: impl Into<NumberMaskingOption>,
    ) {
        if let Err(err) = self
            .response_masks
            .set_number_field_masks(fields.into(), masking_option.into())
        {
            error!(
                "[SpeakeasySDK Internal error] - invalid response field mask string: {}",
                err
            );
        }
    }
}

// private masking functions
#[doc(hidden)]
impl Masking {
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.query_string_mask.is_empty()
            && self.request_header_mask.is_empty()
            && self.response_header_mask.is_empty()
            && self.request_cookie_mask.is_empty()
            && self.response_cookie_mask.is_empty()
            && self.request_masks.is_empty()
            && self.response_masks.is_empty()
    }
}

impl From<Masking> for MaskingMetadata {
    fn from(masking: Masking) -> Self {
        //TODO: finish if this is what MaskingMetadata should look like

        MaskingMetadata {
            request_header_masks: masking.request_header_mask.into(),
            request_cookie_masks: masking.request_cookie_mask.into(),
            request_field_masks_string: HashMap::new(),
            request_field_masks_number: HashMap::new(),
            response_header_masks: masking.response_header_mask.into(),
            response_cookie_masks: masking.response_cookie_mask.into(),
            response_field_masks_string: HashMap::new(),
            response_field_masks_number: HashMap::new(),
            query_string_masks: masking.query_string_mask.into(),
        }
    }
}
