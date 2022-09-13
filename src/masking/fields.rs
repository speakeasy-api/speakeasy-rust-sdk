use std::ops::Deref;

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

impl Deref for Fields {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Fields {
    fn to_vec(self) -> Vec<String> {
        self.0
    }

    fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }
}
