pub(crate) mod attr_definition;
pub(crate) mod element_definition;
pub(crate) mod pre_attribute;

use attr_definition::AttrDefinition;
use element_definition::ElementDefinition;
use pre_attribute::PreAttribute;
use syn::parse::Parse;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Model {
    attribute_definitions: Vec<AttrDefinition>,
    element_definitions: Vec<ElementDefinition>,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut model = Model::default();

        while !input.is_empty() {
            let pre_attribute = PreAttribute::parse(input).map_err(|err| {
                syn::Error::new(
                    err.span(),
                    format!("Failed to parse pre_attribute {err:#?}"),
                )
            })?;
            match pre_attribute {
                PreAttribute::AttrDefinition => {
                    let attr_definition = AttrDefinition::parse(input)?;
                    model.attribute_definitions.push(attr_definition);
                }
                PreAttribute::ElementDefiniton => {
                    let element_definition = ElementDefinition::parse(input)?;
                    model.element_definitions.push(element_definition);
                }
            }
        }
        Ok(model)
    }
}
