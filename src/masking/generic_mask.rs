use super::{fields::FieldsSearchMap, Fields, StringMaskingOption};
use std::{collections::HashMap, marker::PhantomData};

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

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_none()
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

    fn mask<'a>(&'a self, field: &str, value: &'a str) -> &str {
        // If the field is not in the list of fields to mask, return the value as is.
        if let Some(field_index) = self.fields.get(field) {
            self.mask_option.get_mask_replacement(field, field_index)
        } else {
            value
        }
    }
}

impl<T> From<GenericMask<T>> for HashMap<String, String> {
    fn from(mask: GenericMask<T>) -> Self {
        match mask.0 {
            Some(inner) => inner.into(),
            None => HashMap::new(),
        }
    }
}

impl<T> From<GenericMaskInner<T>> for HashMap<String, String> {
    fn from(mask: GenericMaskInner<T>) -> Self {
        mask.fields
            .into_iter()
            .map(|(field, index)| {
                let value = mask
                    .mask_option
                    .get_mask_replacement(&field, index)
                    .to_string();

                (field, value)
            })
            .collect()
    }
}
