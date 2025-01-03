use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::PathArguments;

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
                attributes: Vec<bevy_spts_dioxus::StaticTemplateAttribute>,
            },}
        })
        .collect();

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum SptsDioxusAdapter {
            #variants

            Dynamic { id: usize },
        }

        pub type Hooks = SptsDioxusHooks<SptsDioxusAdapter>;
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
                    namespace: Some("bevy_spts_dioxus"),
                    attrs,
                    children,
                } => {
                    let children = children.iter().map(Self::from_dioxus).collect();
                    let attributes: Vec<bevy_spts_dioxus::StaticTemplateAttribute> = attrs.
                        iter()
                        .filter_map(|v| v.try_into().ok())
                        .collect();
                    Self::#element_ident { children, attributes }
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
                    core::panic!("bevy_spts_dioxus: Unknown dioxus element '{tag}' with namespace {namespace:?}.")
                }

                other => {
                    core::panic!("bevy_spts_dioxus: Unsupported dioxus node {other:?}.")
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

                        let last_segment = component_type.path.segments.last().unwrap();
                        // The identifier for the type, like `Handle`
                        let ident = &last_segment.ident;

                        // Check if there are any generic arguments, like `<Mesh>`
                        let ufc_generic_args =
                            if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
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
                Self::#element_ident { children, attributes } => {
                    let children = children
                        .iter()
                        .map(|child| child.spawn(world))
                        .collect::<Box<[_]>>();

                    use bevy_spts_dioxus::SptsDioxusElement;
                    let mut entity_mut = dioxus_elements::#element_ident::spawn(world);
                    #insert_components
                    entity_mut.add_children(&children);
                    let entity = entity_mut.id();
                    // Apply static attributes
                    for attr in attributes {
                        let value = dioxus_core::AttributeValue::Text(attr.value.into());
                        Self::apply_attribute(world, entity, attr.name, &value);
                    }
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
                        Transform::default(),
                        Visibility::default(),
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
                    .unwrap_or_else(|| panic!("bevy_spts_dioxus: While applying component attr '{}', couldn't downcast to type '{}'.", stringify!(#field_ident), stringify!(#component_type)))
                    .clone();
                let mut entity_mut = world.entity_mut(entity);
                let mut current_value = entity_mut
                    .get_mut::<#component_type>()
                    .unwrap_or_else(|| panic!("bevy_spts_dioxus: While applying component attr '{}', couldn't get component '{}' on entity '{entity:?}' to mutate.", stringify!(#field_ident), stringify!(#component_type)));
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

                unknown => core::panic!("bevy_spts_dioxus: Unexpected attribute '{unknown}'."),
            }
        }
    }
}

pub fn implement_template_node(model: &Model) -> TokenStream {
    let from_dioxus = implement_from_dioxus(model);

    let spawn = implement_spawn(model);

    let apply_attribute = implement_apply_attribute(model);

    quote! {
        impl bevy_spts_dioxus::SptsDioxusTemplateNode for SptsDioxusAdapter {
            #from_dioxus

            #spawn

            #apply_attribute
        }
    }
}
