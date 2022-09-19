use super::{fields::FieldsSearchMap, Fields, StringMaskingOption};
use std::marker::PhantomData;

#[derive(Debug, Clone, Default)]
pub struct QueryStringMask;
#[derive(Debug, Clone, Default)]
pub struct RequestHeaderMask;
#[derive(Debug, Clone, Default)]
pub struct ResponseHeaderMask;
#[derive(Debug, Clone, Default)]
pub struct RequestCookieMask;
#[derive(Debug, Clone, Default)]
pub struct ResponseCookieMask;

#[derive(Debug, Clone, Default)]
pub(crate) struct GenericMask<T>(Option<GenericMaskInner<T>>);

impl<T> GenericMask<T> {
    pub(crate) fn new(fields: Fields, mask_option: StringMaskingOption) -> Self {
        let inner = GenericMaskInner::new(fields, mask_option);
        Self(Some(inner))
    }

    pub(crate) fn mask(&self, field: &str, value: &str) -> String {
        match &self.0 {
            Some(inner) => inner.mask(field, value).to_string(),
            None => value.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenericMaskInner<T> {
    phantom: PhantomData<T>,
    fields: FieldsSearchMap,
    mask_option: StringMaskingOption,
}

impl<T> GenericMaskInner<T> {
    pub fn new(fields: Fields, mask_option: StringMaskingOption) -> Self {
        Self {
            phantom: PhantomData,
            fields: fields.into(),
            mask_option,
        }
    }

    fn mask(&self, field: &str, value: &str) -> &str {
        self.mask_option
            .get_mask_replacement(field, self.fields.get(field))
    }
}
