use syn::{braced, parse::Parse, Ident, Token};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ElementDefinition {
    pub ident: Ident,
    pub components: Vec<Ident>,
    pub attributes: Vec<Ident>,
}

impl Parse for ElementDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;


        let mut result = ElementDefinition {
            ident,
            components: vec![],
            attributes: vec![],
        };

        let content;
        braced!(content in input);

        while !content.is_empty() {
            let component_or_attribute_ident = content.parse::<Ident>()?;

            let string = component_or_attribute_ident.to_string();
            let Some(first_char) = string.chars().next() else {
                return Err(syn::Error::new(component_or_attribute_ident.span(), "Element can not have an zero length identifier.  This should be impossible so please contact me if this ever occurs."));
            };

            if first_char.is_uppercase() {
                result.components.push(component_or_attribute_ident);
            } else {
                result.attributes.push(component_or_attribute_ident);
            }

            content.parse::<Token![,]>()?;
        }

        Ok(result)
    }
}
