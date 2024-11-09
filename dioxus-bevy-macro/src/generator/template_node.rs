use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

use crate::parser::{element_definition::ElementAttribute, Model};

/// Generates the template node enum
///
/// * `model`:
pub fn generate_template_node(model: &Model) -> TokenStream {
    let variants: TokenStream = model
        .dioxus_elements_module
        .element_definitions
        .iter()
        .map(|el_def| {
            let ident = &el_def.ident;
            quote! { #ident {
                children: Box<[Self]>,
            }}
        })
        .collect();

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, PartialEq)]
        pub enum DioxusBevyAdapter {
            #variants
        }
    }
}

fn implement_from_dioxus(model: &Model) -> TokenStream {
    let defined_element_matches: TokenStream = model
        .dioxus_elements_module
        .element_definitions
        .iter()
        .map(|el_def| {
            let element_ident = &el_def.ident;
            quote! {
                dioxus_core::TemplateNode::Element {
                    tag: stringify!(#element_ident),
                    namespace: Some("dioxus_bevy"),
                    attrs: _,
                    children,
                } => {
                    let children = children.iter().map(Self::from_dioxus).collect();

                    Self::#element_ident { children }
                }
            }
        })
        .collect();

    quote! {
        fn from_dioxus(node: &dioxus_core::TemplateNode) -> Self {
            match node {
                #defined_element_matches

                dioxus_core::TemplateNode::Element {
                    tag,
                    namespace,
                    attrs,
                    children,
                } => {
                    core::panic!("dioxus_bevy: Unknown dioxus element '{tag}' with namespace {namespace:?}.")
                }

                other => {
                    core::panic!("dioxus_bevy: Unsupported dioxus node {other:?}.")
                }
            }
        }
    }
}

fn implement_spawn(model: &Model) -> TokenStream {
    let variant_matches: TokenStream = model
        .dioxus_elements_module
        .element_definitions
        .iter()
        .map(|el_def| {
            let element_ident = &el_def.ident;

            quote! {
                Self::#element_ident { children } => {
                    let children = children
                        .iter()
                        .map(|child| child.spawn(world))
                        .collect::<Box<[_]>>();

                    use dioxus_bevy::DioxusBevyElement;
                    dioxus_elements::#element_ident::spawn(world).push_children(&children).id()
                }
            }
        })
        .collect();

    quote! {
        fn spawn(&self, world: &mut World) -> Entity {
            match self {
                #variant_matches
            }
        }
    }
}

fn implement_apply_attribute(model: &Model) -> TokenStream {
    let all_attributes: HashSet<&ElementAttribute> = model
        .dioxus_elements_module
        .element_definitions
        .iter()
        .flat_map(|el_def| &el_def.attributes)
        .collect();
    let attribute_matches: TokenStream = all_attributes
        .into_iter()
        .map(|el_attribute| {
            let ElementAttribute { field_ident, handler_ident } = el_attribute;
            quote! { stringify!(#field_ident) => #handler_ident(entity_mut, value), }
        })
        .collect();

    quote! {
        fn apply_attribute(
            mut entity_mut: EntityWorldMut,
            name: &'static str,
            value: &dioxus_core::AttributeValue,
        ) {
            match name {
                #attribute_matches

                unknown => core::panic!("dioxus_bevy: Unexpected attribute '{unknown}'."),
            }
        }
    }
}

pub fn implement_template_node(model: &Model) -> TokenStream {
    let from_dioxus = implement_from_dioxus(model);

    let spawn = implement_spawn(model);

    let apply_attribute = implement_apply_attribute(model);

    quote! {
        impl dioxus_bevy::DioxusBevyTemplateNode for DioxusBevyAdapter {
            #from_dioxus

            #spawn

            #apply_attribute
        }
    }
}
