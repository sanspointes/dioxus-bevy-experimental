use dioxus::{dioxus_core::AttributeValue, prelude::TemplateAttribute};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticTemplateAttribute {
    pub name: &'static str,
    pub value: &'static str,
    pub namespace: Option<&'static str>,
}

pub enum FromTemplateAttributeError {
    NotStatic,
}

impl TryFrom<&TemplateAttribute> for StaticTemplateAttribute {
    type Error = FromTemplateAttributeError;
    fn try_from(value: &TemplateAttribute) -> Result<Self, Self::Error> {
        match value {
            TemplateAttribute::Static { name, value, namespace } => Ok(StaticTemplateAttribute { name, value, namespace: *namespace }),
            TemplateAttribute::Dynamic { .. } => Err(FromTemplateAttributeError::NotStatic),
        }
    }
}

impl From<&StaticTemplateAttribute> for AttributeValue {
    fn from(value: &StaticTemplateAttribute) -> Self {
        Self::Text(value.value.to_string())
    }
}
