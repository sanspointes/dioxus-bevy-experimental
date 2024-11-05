use syn::{braced, parse::Parse, spanned::Spanned, Attribute, Ident, Token};

#[derive(Debug, Clone, Copy)]
pub enum ElementAttribute {
    Attribute,
    Component,
}

const UNEXPECTED_ERROR_MSG: &str =
    "dixoxus_bevy: Expected attributes `#[attr]`, or `#[component]`.";

impl Parse for ElementAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let rust_attribute =
            Attribute::parse_outer(input).map_err(|_err| input.error(UNEXPECTED_ERROR_MSG))?;

        println!("rust_attribute: {rust_attribute:#?}");

        let first = rust_attribute
            .first()
            .ok_or(input.error(UNEXPECTED_ERROR_MSG))?;
        dbg!(&first);
        let first_path = first.path();
        dbg!(&first_path);
        if first_path.is_ident("attr") {
            Ok(ElementAttribute::Attribute)
        } else if first_path.is_ident("component") {
            Ok(ElementAttribute::Component)
        } else {
            Err(input.error(UNEXPECTED_ERROR_MSG))
        }
    }
}

#[derive(Debug, Default)]
pub struct ElementDefinition {
    components: Vec<Ident>,
    attributes: Vec<Ident>,
}

impl Parse for ElementDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::token::Struct>()?;
        let element_ident = input
            .parse::<Ident>()
            .map_err(|err| syn::Error::new(err.span(), "Expected element name."))?;

        let element_ident_str = format!("{element_ident}");
        let has_capitals = element_ident_str.chars().any(|char| char.is_uppercase());
        if has_capitals {
            return Err(syn::Error::new(
                element_ident.span(),
                "Element name must be lowercase.",
            ));
        };

        let inner_content;
        braced!(inner_content in input);
        dbg!(&inner_content);

        let mut model = ElementDefinition {
            components: vec![],
            attributes: vec![],
        };

        while !inner_content.is_empty() {
            let element_attribute = ElementAttribute::parse(&inner_content)?;
            println!("Parsing ElementDefinition found rust attribute {element_attribute:?}");
            match element_attribute {
                ElementAttribute::Attribute => {
                    let attr = inner_content.parse::<Ident>()?;
                    println!("Found attr {attr:?}");
                    inner_content.parse::<Token![,]>()?;
                    model.attributes.push(attr);
                }
                ElementAttribute::Component => {
                    let comp = inner_content.parse::<Ident>()?;
                    println!("Found comp {comp:?}");
                    inner_content.parse::<Token![,]>()?;
                    model.components.push(comp);
                }
            }
        }

        Ok(model)
    }
}
