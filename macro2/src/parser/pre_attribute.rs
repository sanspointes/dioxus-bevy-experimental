//! Contains logic for parsing the initial #[define_attr] #[define_element], etc attributes.

use syn::{parse::Parse, spanned::Spanned, Attribute};

#[derive(Debug, Clone, Copy)]
pub enum PreAttribute {
    AttrDefinition,
    ElementDefiniton,
}

const UNEXPECTED_ERROR_MSG: &str = "dixoxus_bevy: Expected attributes `#[define_attr]`, or `#[define_element]`.";

impl Parse for PreAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let rust_attribute = Attribute::parse_outer(input).map_err(|err| {
            syn::Error::new(err.span(), UNEXPECTED_ERROR_MSG)
        })?;
        println!("rust_attribute: {rust_attribute:#?}");
        let first = rust_attribute.first().ok_or(syn::Error::new(input.span(), UNEXPECTED_ERROR_MSG))?;
        println!("first: {first:#?}");
        let first_path = first.path();
        if first_path.is_ident("define_attr") {
            Ok(PreAttribute::AttrDefinition)
        } else if first_path.is_ident("define_element") {
            Ok(PreAttribute::ElementDefiniton)
        } else {
            println!("first_path: {first_path:#?}");
            Err(syn::Error::new(first.span(), UNEXPECTED_ERROR_MSG))
        }
    }
}
