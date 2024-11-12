use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{PathArguments, Type};

use crate::parser::{
    element_definition::{ElementAttribute, ElementComponent},
    Model,
};

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
            },}
        })
        .collect();

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum DioxusBevyAdapter {
            #variants

            Dynamic { id: usize },
        }

        pub type Hooks = DBHooks<DioxusBevyAdapter>;
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

                dioxus_core::TemplateNode::Dynamic { id } => {
                    Self::Dynamic { id: *id }
                }

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

            let insert_components: TokenStream = if !el_def.components.is_empty() {
                let component_defaults: TokenStream = el_def
                    .components
                    .iter()
                    .map(|component_ident| {
                        let ElementComponent { component_type, .. } = component_ident;

                        println!("implement_spawn: {component_type:?}");

                        let last_segment = component_type.path.segments.last().unwrap();
                        // The identifier for the type, like `Handle`
                        let ident = &last_segment.ident;

                        // Check if there are any generic arguments, like `<Mesh>`
                        let ufc_generic_args = if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            let args = &args.args;
                            quote! {::<#args>}
                        } else {
                            TokenStream::new()
                        };

                        let mut joined = ident.to_token_stream();
                        joined.extend(ufc_generic_args);

                        // Generate the final UFC syntax, e.g., `Handle::<Mesh>`
                        quote! {
                            #joined::default(),
                        }
                    })
                    .collect();
                quote! { entity_mut.insert((#component_defaults)); }
            } else {
                TokenStream::new()
            };

            quote! {
                Self::#element_ident { children } => {
                    let children = children
                        .iter()
                        .map(|child| child.spawn(world))
                        .collect::<Box<[_]>>();

                    use dioxus_bevy::DioxusBevyElement;
                    let mut entity_mut = dioxus_elements::#element_ident::spawn(world);
                    #insert_components
                    entity_mut.push_children(&children);
                    let entity = entity_mut.id();
                    ;
                    entity
                }
            }
        })
        .collect();

    quote! {
        fn spawn(&self, world: &mut World) -> Entity {
            match self {
                #variant_matches

                Self::Dynamic { id } => {
                    world.spawn((
                        Name::from("Dynamic"),
                        SpatialBundle::default(),
                    )).id()
                }
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
            let ElementAttribute {
                field_ident,
                handler_ident,
            } = el_attribute;
            quote! { stringify!(#field_ident) => #handler_ident(world, entity, value), }
        })
        .collect();

    let all_components: HashSet<&ElementComponent> = model
        .dioxus_elements_module
        .element_definitions
        .iter()
        .flat_map(|el_def| &el_def.components)
        .collect();
    let component_matches: TokenStream = all_components
        .into_iter()
        .map(|el_attribute| {
            let ElementComponent {
                field_ident,
                component_type,
            } = el_attribute;
            quote! { stringify!(#field_ident) => {
                let value = value
                    .as_concrete::<#component_type>()
                    .unwrap_or_else(|| panic!("dioxus_bevy: While applying component attr '{}', couldn't downcast to type '{}'.", stringify!(#field_ident), stringify!(#component_type)))
                    .clone();
                let mut entity_mut = world.entity_mut(entity);
                let mut current_value = entity_mut
                    .get_mut::<#component_type>()
                    .unwrap_or_else(|| panic!("dioxus_bevy: While applying component attr '{}', couldn't get component '{}' on entity '{entity:?}' to mutate.", stringify!(#field_ident), stringify!(#component_type)));
                *current_value = value;
            } }
        })
        .collect();

    quote! {
        fn apply_attribute(
            world: &mut World,
            entity: Entity,
            name: &'static str,
            value: &dioxus_core::AttributeValue,
        ) {
            println!("Applying attribute {name} with {value:?}.");
            match name {
                #attribute_matches
                #component_matches

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
