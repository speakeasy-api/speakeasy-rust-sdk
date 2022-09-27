use std::{collections::HashMap, ops::Deref};

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

impl From<Vec<&str>> for Fields {
    fn from(fields: Vec<&str>) -> Self {
        Self(fields.iter().map(ToString::to_string).collect())
    }
}

impl From<Fields> for Vec<String> {
    fn from(fields: Fields) -> Self {
        fields.0
    }
}

impl Deref for Fields {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BodyMaskFieldsSearchMap(HashMap<String, (String, usize)>);

impl From<Fields> for BodyMaskFieldsSearchMap {
    fn from(fields: Fields) -> Self {
        Self(
            fields
                .iter()
                .enumerate()
                .map(|(i, field)| (format!("\"{}\"", &field), (field.clone(), i)))
                .collect(),
        )
    }
}

impl BodyMaskFieldsSearchMap {
    pub(crate) fn get(&self, field: &str) -> Option<(String, usize)> {
        self.0.get(field).cloned()
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (String, (String, usize))> {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GenericMaskFieldsSearchMap(HashMap<String, usize>);

impl From<Fields> for GenericMaskFieldsSearchMap {
    fn from(fields: Fields) -> Self {
        Self(
            fields
                .iter()
                .enumerate()
                .map(|(i, field)| (field.clone(), i))
                .collect(),
        )
    }
}

impl GenericMaskFieldsSearchMap {
    pub(crate) fn get(&self, field: &str) -> Option<usize> {
        self.0.get(field).copied()
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (String, usize)> {
        self.0.into_iter()
    }
}
