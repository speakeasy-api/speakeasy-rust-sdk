use std::collections::HashMap;

use super::{DEFAULT_NUMBER_MASK, DEFAULT_STRING_MASK};

#[derive(Debug, Clone)]
pub enum StringMaskingOption {
    None,
    SingleMask(String),
    MultipleMasks(Vec<String>),
    AssociatedMasks(HashMap<String, String>),
}

impl Default for StringMaskingOption {
    fn default() -> Self {
        Self::None
    }
}

impl From<Option<String>> for StringMaskingOption {
    fn from(maybe_mask: Option<String>) -> Self {
        match maybe_mask {
            Some(mask) => StringMaskingOption::SingleMask(mask),
            None => StringMaskingOption::None,
        }
    }
}

impl From<String> for StringMaskingOption {
    fn from(mask: String) -> Self {
        StringMaskingOption::SingleMask(mask)
    }
}

impl From<&str> for StringMaskingOption {
    fn from(mask: &str) -> Self {
        StringMaskingOption::SingleMask(mask.to_string())
    }
}

impl From<Vec<String>> for StringMaskingOption {
    fn from(masks: Vec<String>) -> Self {
        StringMaskingOption::MultipleMasks(masks)
    }
}

impl From<&[&str]> for StringMaskingOption {
    fn from(masks: &[&str]) -> Self {
        StringMaskingOption::MultipleMasks(masks.iter().map(ToString::to_string).collect())
    }
}

impl From<Vec<&str>> for StringMaskingOption {
    fn from(masks: Vec<&str>) -> Self {
        StringMaskingOption::MultipleMasks(masks.iter().map(ToString::to_string).collect())
    }
}

impl From<HashMap<String, String>> for StringMaskingOption {
    fn from(masks: HashMap<String, String>) -> Self {
        StringMaskingOption::AssociatedMasks(masks)
    }
}

impl From<HashMap<&str, &str>> for StringMaskingOption {
    fn from(masks: HashMap<&str, &str>) -> Self {
        let masks = masks
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        StringMaskingOption::AssociatedMasks(masks)
    }
}

impl StringMaskingOption {
    pub(crate) fn get_mask_replacement<'a>(
        &self,
        field: &'a str,
        maybe_index: Option<usize>,
    ) -> &str {
        match self {
            Self::None => DEFAULT_STRING_MASK,
            Self::SingleMask(mask) => mask,
            Self::MultipleMasks(ref masks) => {
                if let Some(index) = maybe_index {
                    masks
                        .get(index)
                        .map(String::as_str)
                        .unwrap_or(DEFAULT_STRING_MASK)
                } else {
                    DEFAULT_STRING_MASK
                }
            }
            Self::AssociatedMasks(ref masks_map) => masks_map
                .get(field)
                .map(String::as_str)
                .unwrap_or(DEFAULT_STRING_MASK),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NumberMaskingOption {
    None,
    SingleMask(i32),
    MultipleMasks(Vec<i32>),
    AssociatedMasks(HashMap<String, i32>),
}

impl Default for NumberMaskingOption {
    fn default() -> Self {
        Self::None
    }
}

impl From<Option<i32>> for NumberMaskingOption {
    fn from(maybe_mask: Option<i32>) -> Self {
        match maybe_mask {
            Some(mask) => NumberMaskingOption::SingleMask(mask),
            None => NumberMaskingOption::None,
        }
    }
}

impl From<i32> for NumberMaskingOption {
    fn from(mask: i32) -> Self {
        NumberMaskingOption::SingleMask(mask)
    }
}

impl From<&[i32]> for NumberMaskingOption {
    fn from(masks: &[i32]) -> Self {
        NumberMaskingOption::MultipleMasks(masks.to_vec())
    }
}

impl From<Vec<i32>> for NumberMaskingOption {
    fn from(masks: Vec<i32>) -> Self {
        NumberMaskingOption::MultipleMasks(masks)
    }
}

impl From<HashMap<String, i32>> for NumberMaskingOption {
    fn from(masks: HashMap<String, i32>) -> Self {
        NumberMaskingOption::AssociatedMasks(masks)
    }
}

impl From<HashMap<&str, i32>> for NumberMaskingOption {
    fn from(masks: HashMap<&str, i32>) -> Self {
        NumberMaskingOption::AssociatedMasks(
            masks.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        )
    }
}

impl NumberMaskingOption {
    pub(crate) fn get_mask_replacement(&self, field: &str, maybe_index: Option<usize>) -> i32 {
        match self {
            Self::None => DEFAULT_NUMBER_MASK,
            Self::SingleMask(mask) => *mask,
            Self::MultipleMasks(ref masks) => {
                if let Some(index) = maybe_index {
                    masks.get(index).copied().unwrap_or(DEFAULT_NUMBER_MASK)
                } else {
                    DEFAULT_NUMBER_MASK
                }
            }
            Self::AssociatedMasks(ref masks_map) => {
                masks_map.get(field).copied().unwrap_or(DEFAULT_NUMBER_MASK)
            }
        }
    }
}
