use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parser::Model;

// TODO: TAG_NAME must be moved into a submodule called "elements" with an impl the same name.
pub fn generate_dioxus_elements_tag_names(model: &Model) -> TokenStream {
    let elements: TokenStream = model.dioxus_elements_module.element_definitions.iter().map(|el_defininition| {
        let el_ident = &el_defininition.ident;
        quote! {
            #[allow(non_camel_case_types)]
            pub struct #el_ident;
            #[allow(non_upper_case_globals)]
            impl #el_ident {
                pub const TAG_NAME: &'static str = stringify!(#el_ident);
            }
        }
    }).collect();

    quote! {
        pub mod elements {
            #elements
        }
    }
}

pub fn generate_dioxus_elements(model: &Model) -> TokenStream {
    let elements: TokenStream = model.dioxus_elements_module.element_definitions.iter().map(|el_defininition| {
        let element_attributes: TokenStream = el_defininition.attributes.iter().map(|el_attribute| {
            let field_ident = &el_attribute.field_ident;
            quote! { pub const #field_ident: AttributeDescription = (stringify!(#field_ident), None, false); }
        }).collect();

        let component_attributes: TokenStream = el_defininition.components.iter().map(|el_component| {
            let field_ident = &el_component.field_ident;
            quote! { pub const #field_ident: AttributeDescription = (stringify!(#field_ident), None, false); }
        }).collect();

        let el_ident = &el_defininition.ident;
        quote! {
            #[allow(non_camel_case_types)]
            pub struct #el_ident;
            #[allow(non_upper_case_globals)]
            impl #el_ident {
                pub const NAME_SPACE: Option<&'static str> = NAME_SPACE;

                pub const entity: AttributeDescription = ("entity", None, false);

                #element_attributes
                #component_attributes
            }
        }
    }).collect();

    let pass_through_items: TokenStream = model
        .dioxus_elements_module
        .pass_through_items
        .iter()
        .map(|v| v.to_token_stream())
        .collect();

    let element_tag_names = generate_dioxus_elements_tag_names(model);

    quote! {
        pub mod dioxus_elements {
            #pass_through_items

            pub type AttributeDescription = (&'static str, Option<&'static str>, bool);
            const NAME_SPACE: Option<&'static str> = Some("bevy_spts_dioxus");

            #element_tag_names

            #elements
        }
    }
}
