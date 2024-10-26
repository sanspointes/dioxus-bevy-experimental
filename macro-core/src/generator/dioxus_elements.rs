use proc_macro2::TokenStream;
use quote::quote;

use crate::parser::{
    element_definition::ElementDefinition,
    Model,
};

/// Generates the `dioxus_elements` module, which is used as the back-bone for auto-complete
/// in the JSX macro.
///
/// * `model`: 
pub fn generate_elements(model: &Model) -> proc_macro2::TokenStream {
    let element_tokens: TokenStream = model.elements.values().map(|element| {
        let ElementDefinition {
            ident,
            components,
            attributes,
        } = element;

        let attributes: TokenStream = attributes
            .iter()
            .map(|attr_ident| {
                quote! {pub const #attr_ident: AttributeDescription = ("#attr_ident", None, false);}
            })
            .collect();

        let components: TokenStream = components
            .iter()
            .map(|comp_ident| {
                quote! {pub const #comp_ident: AttributeDescription = ("#comp_ident", None, false);}
            })
            .collect();

        quote! {
            #[allow(non_camel_case_types)]
            pub struct #ident;
            #[allow(non_upper_case_globals)]
            impl #ident {
                pub const TAG_NAME: &'static str = "#ident";
                pub const NAME_SPACE: Option<&'static str> = NAME_SPACE;

                #attributes
                #components
            }
        }
    }).collect();

    quote! {
        pub mod dioxus_elements {
            pub type AttributeDescription = (&'static str, Option<&'static str>, bool);
            const NAME_SPACE: Option<&'static str> = Some("bevy_ui");

            #element_tokens
        }
    }
}
