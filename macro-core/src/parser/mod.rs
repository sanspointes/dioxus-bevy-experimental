mod prop_definition;
mod element_definition;

use std::collections::HashMap;

use element_definition::ElementDefinition;
use prop_definition::{PropDefinition, RendererAttribute};
use syn::{parse::Parse, Ident, Token};

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Model {
    props: HashMap<String, PropDefinition>,
    elements: HashMap<String, ElementDefinition>,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut model = Model::default();

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![@]) {
                match input.parse::<RendererAttribute>()? {
                    RendererAttribute::PropDefinition(prop_definition) => {
                        model
                            .props
                            .insert(prop_definition.ident.to_string(), prop_definition);
                    }
                }
                // Parse renderer attribute
            } else if lookahead.peek(Ident) {
                let element_definition = input.parse::<ElementDefinition>()?;
                model.elements.insert(element_definition.ident.to_string(), element_definition);
                // Parse element definition
            } else {
                return Err(syn::Error::new(input.span(), "Unexpected input."));
            }
        }
        Ok(model)
    }
}
