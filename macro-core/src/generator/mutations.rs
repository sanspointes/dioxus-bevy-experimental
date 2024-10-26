use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

use syn::Ident;

use crate::parser::element_definition::ElementDefinition;
use crate::parser::Model;
use crate::parser::prop_definition::PropDefinition;

fn generate_attr_match(attr_definition: &PropDefinition) -> syn::Result<proc_macro2::TokenStream> {
    let internal_ident = attr_definition.internal_ident();
    let PropDefinition { ty, .. } = attr_definition;

    let match_handler = match ty.to_string().as_str() {
        "f64" => quote! {dioxus_core::nodes::AttributeValue::Float(value)},
        "String" => quote! {dioxus_core::nodes::AttributeValue::String(value)},
        "bool" => quote! {dioxus_core::nodes::AttributeValue::Bool(value)},
        "i64" => quote! {dioxus_core::nodes::AttributeValue::Int(value)},
        other => return Err(syn::Error::new(ty.span(), format!("Unknown prop type '{other}', expected f64, String, bool, or i64."))),
    };

    Ok(quote! {
        "#ident" => {
            match value {
                #match_handler => #internal_ident(entity_mut, value),
                dioxus_core::nodes::AttributeValue::None => todo!("Handle removing attributes."),
                other => {},
            }
        },
    })
}

fn generate_component_match(component_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        "#component_ident" => self.apply_required_component::<#component_ident>(entity_mut, value),
    }
}

fn generate_mutation_applier_set_attribute(model: &Model) -> syn::Result<proc_macro2::TokenStream> {
    let attr_matches: syn::Result<proc_macro2::TokenStream> = model
        .props
        .values()
        .map(generate_attr_match)
        .collect();
    let attr_matches = attr_matches?;

    let mut components: HashSet<Ident> = HashSet::new();
    for element in model.elements.values() {
        components.extend(element.components.clone());
    }

    let component_matches: proc_macro2::TokenStream = components
        .iter()
        .map(generate_component_match)
        .collect();

    Ok(quote! {
        fn set_attribute(
                &mut self,
                name: &'static str,
                ns: Option<&'static str>,
                value: &dioxus_core::nodes::AttributeValue,
                id: dioxus_core::ElementId,
            ) {

            let entity = self.el_to_entity[&id];
            let entity_mut = self.world.entity_mut(entity);

            println!("Setting attribute '{name}' to '{value:?}");

            match name {
                #attr_matches

                #component_matches

                other => {
                    let stringified_namespace = ns.unwrap_or("dioxus_bevy");
                    warn!("{stringified_namespace}: Received unknown attribute {other:?} with value {value:?}");
                },
            }
        }
    })
}

fn generate_mutation_applier(model: &Model) -> syn::Result<proc_macro2::TokenStream> {
    let set_attribute_tokens = generate_mutation_applier_set_attribute(model)?;
    let tokens = quote! {
        pub struct MutationApplier<'a> {
            el_to_entity: &'a mut bevy::utils::HashMap<dioxus_core::ElementId, Entity>,
            entity_to_el: &'a mut bevy::ecs::entity::EntityHashMap<dioxus_core::ElementId>,
            templates: &'a mut bevy::utils::HashMap<String, BevyTemplate>,
            world: &'a mut World,
            stack: Vec<Entity>,
        }

        impl<'a> MutationApplier<'a> {
            pub fn new(
                el_to_entity: &'a mut bevy::utils::HashMap<dioxus_core::ElementId, Entity>,
                entity_to_el: &'a mut bevy::ecs::entity::EntityHashMap<dioxus_core::ElementId>,
                templates: &'a mut bevy::utils::HashMap<String, BevyTemplate>,
                root_entity: Entity,
                world: &'a mut World,
            ) -> Self {
                el_to_entity.insert(dioxus_core::ElementId(0), root_entity);
                entity_to_el.insert(root_entity, dioxus_core::ElementId(0));

                Self {
                    el_to_entity,
                    entity_to_el,
                    templates,
                    world,
                    stack: vec![root_entity],
                }
            }
        }

        impl<'a> MutationApplier<'a> {
            pub fn despawn_recursive(&mut self, entity: Entity) {
                let mut ss: bevy::ecs::system::SystemState<Query<&Children>> = SystemState::new(self.world);
                let query_children = ss.get_mut(self.world);
                for child in query_children.iter_descendants(entity) {
                    if let Some(existing_element_id) = self.entity_to_el.remove(&child) {
                        self.el_to_entity
                            .remove(&existing_element_id);
                    }
                }

                if let Some(existing_element_id) = self.entity_to_el.remove(&entity) {
                    self.el_to_entity
                        .remove(&existing_element_id);
                }

                DespawnRecursive { entity }.apply(self.world);
            }

            pub fn apply_required_component<T: bevy::prelude::Component + Clone + Default>(&mut self, entity: Entity, value: &dioxus_core::nodes::AttributeValue) {
                match value {
                    dioxus_core::nodes::AttributeValue::Any(boxed_any) => {
                        let Some(component) = boxed_any.as_any().downcast_ref::<T>() else {
                            warn!("apply_any_component: Tried to downcast into but the provided any pointer wasn't of this type.");
                            return;
                        };
                        let mut entity_mut = self.world.entity_mut(entity);
                        entity_mut.insert(component.clone());
                    },
                    dioxus_core::nodes::AttributeValue::None => {
                        let mut entity_mut = self.world.entity_mut(entity);
                        entity_mut.insert(T::default());
                    },
                    other => warn!("Incorrect value passed to 'transform' attribute. Expected 'Transform' but found {other:?}."),
                }
            }

            pub fn apply_optional_component<T: bevy::prelude::Component + Clone + Default>(&mut self, entity: Entity, value: &dioxus_core::nodes::AttributeValue) {
                match value {
                    dioxus_core::nodes::AttributeValue::Any(boxed_any) => {
                        let Some(component) = boxed_any.as_any().downcast_ref::<T>() else {
                            warn!("apply_any_component: Tried to downcast into but the provided any pointer wasn't of this type.");
                            return;
                        };
                        let mut entity_mut = self.world.entity_mut(entity);
                        entity_mut.insert(component.clone());
                    },
                    dioxus_core::nodes::AttributeValue::None => {
                        let mut entity_mut = self.world.entity_mut(entity);
                        entity_mut.remove::<T>();
                    },
                    other => warn!("Incorrect value passed to 'transform' attribute. Expected 'Transform' but found {other:?}."),
                }
            }
        }

        impl<'a> dioxus_core::WriteMutations for MutationApplier<'a> {
            fn register_template(&mut self, template: dioxus_core::Template) {
                self.templates.insert(
                    template.name.to_owned(),
                    BevyTemplate::from_dioxus(&template),
                );
            }

            fn append_children(&mut self, id: dioxus_core::ElementId, m: usize) {
                let mut parent = self
                    .world
                    .entity_mut(self.el_to_entity[&id]);
                for child in self.stack.drain((self.stack.len() - m)..) {
                    parent.add_child(child);
                }
            }

            fn assign_node_id(&mut self, path: &'static [u8], id: dioxus_core::ElementId) {
                let mut entity = *self.stack.last().unwrap();
                for index in path {
                    entity = self.world.entity(entity).get::<Children>().unwrap()[*index as usize];
                }
                self.el_to_entity.insert(id, entity);
                self.entity_to_el.insert(entity, id);
            }

            fn create_placeholder(&mut self, id: dioxus_core::ElementId) {
                let entity = self.world.spawn_empty().id();
                self.el_to_entity.insert(id, entity);
                self.entity_to_el.insert(entity, id);
                self.stack.push(entity);
            }

            fn create_text_node(&mut self, value: &str, id: dioxus_core::ElementId) {
                todo!("create_text_node");
                // let entity =
                //     BevyTemplateNode::IntrinsicTextNode(Text::from_section(value, TextStyle::default()))
                //         .spawn(self.world);
                // self.el_to_entity.insert(id, entity);
                // self.entity_to_el.insert(entity, id);
                // self.stack.push(entity);
            }

            fn hydrate_text_node(&mut self, path: &'static [u8], value: &str, id: dioxus_core::ElementId) {
                todo!("hydrate_text_node");
                // let mut entity = *self.stack.last().unwrap();
                // for index in path {
                //     entity = self.world.entity(entity).get::<Children>().unwrap()[*index as usize];
                // }
                // self.world.entity_mut(entity).insert((
                //     Text::from_section(value, TextStyle::default()),
                //     TextLayoutInfo::default(),
                //     TextFlags::default(),
                //     ContentSize::default(),
                // ));
                // self.el_to_entity.insert(id, entity);
                // self.entity_to_el.insert(entity, id);
            }

            fn load_template(&mut self, name: &'static str, index: usize, id: dioxus_core::ElementId) {
                let entity = self.templates[name].roots[index].spawn(self.world);
                self.el_to_entity.insert(id, entity);
                self.entity_to_el.insert(entity, id);
                self.stack.push(entity);
            }

            fn replace_node_with(&mut self, id: dioxus_core::ElementId, m: usize) {
                let existing = self.el_to_entity[&id];
                let existing_parent = self.world.entity(existing).get::<Parent>().unwrap().get();
                let mut existing_parent = self.world.entity_mut(existing_parent);

                let existing_index = existing_parent
                    .get::<Children>()
                    .unwrap()
                    .iter()
                    .position(|child| *child == existing)
                    .unwrap();
                existing_parent
                    .insert_children(existing_index, &self.stack.split_off(self.stack.len() - m));

                self.despawn_recursive(existing);
            }

            fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
                let mut existing = self.stack[self.stack.len() - m - 1];
                for index in path {
                    existing = self.world.entity(existing).get::<Children>().unwrap()[*index as usize];
                }
                let existing_parent = self.world.entity(existing).get::<Parent>().unwrap().get();
                let mut existing_parent = self.world.entity_mut(existing_parent);

                let existing_index = existing_parent
                    .get::<Children>()
                    .unwrap()
                    .iter()
                    .position(|child| *child == existing)
                    .unwrap();
                existing_parent
                    .insert_children(existing_index, &self.stack.split_off(self.stack.len() - m));

            }

            fn insert_nodes_after(&mut self, id: dioxus_core::ElementId, m: usize) {
                let entity = self.el_to_entity[&id];
                let parent = self.world.entity(entity).get::<Parent>().unwrap().get();
                let mut parent = self.world.entity_mut(parent);
                let index = parent
                    .get::<Children>()
                    .unwrap()
                    .iter()
                    .position(|child| *child == entity)
                    .unwrap();
                parent.insert_children(index + 1, &self.stack.split_off(self.stack.len() - m));
            }

            fn insert_nodes_before(&mut self, id: dioxus_core::ElementId, m: usize) {
                let existing = self.el_to_entity[&id];
                let parent = self.world.entity(existing).get::<Parent>().unwrap().get();
                let mut parent = self.world.entity_mut(parent);
                let index = parent
                    .get::<Children>()
                    .unwrap()
                    .iter()
                    .position(|child| *child == existing)
                    .unwrap();
                parent.insert_children(index, &self.stack.split_off(self.stack.len() - m));
            }

            #set_attribute_tokens

            fn set_node_text(&mut self, value: &str, id: dioxus_core::ElementId) {
                todo!("set_node_text");
            }

            fn create_event_listener(&mut self, name: &'static str, id: dioxus_core::ElementId) {
                todo!("create_event_listener");
                // insert_event_listener(
                //     &name,
                //     self.world
                //         .entity_mut(self.el_to_entity[&id]),
                // );
            }

            fn remove_event_listener(&mut self, name: &'static str, id: dioxus_core::ElementId) {
                todo!("remove_event_listener");
                // remove_event_listener(
                //     &name,
                //     self.world
                //         .entity_mut(self.el_to_entity[&id]),
                // );
            }

            fn remove_node(&mut self, id: dioxus_core::ElementId) {
                let entity = self.el_to_entity[&id];
                self.despawn_recursive(entity);
            }

            fn push_root(&mut self, id: dioxus_core::ElementId) {
                self.stack.push(self.el_to_entity[&id]);
            }
        }

        pub struct BevyTemplate {
            roots: Box<[BevyTemplateNode]>,
        }

        impl BevyTemplate {
            fn from_dioxus(template: &dioxus_core::Template) -> Self {
                Self {
                    roots: template
                        .roots
                        .iter()
                        .map(|node| BevyTemplateNode::from_dioxus(node))
                        .collect(),
                }
            }
        }
    };

    Ok(tokens)
}

fn generate_template_node_enum(model: &Model) -> proc_macro2::TokenStream {
    let variants: TokenStream = model.elements.values().map(|element_def| {
        let ElementDefinition { ident, .. } = element_def;
        quote! {
            #ident {
                attributes: Box<[DynamicAttributeId]>,
                children: Box<[Self]>,
            }
        }
    }).collect();

    quote! {
        enum BevyTemplateNode {
            #variants
        }
    }
}

fn generate_template_node_from_dioxus(model: &Model) -> proc_macro2::TokenStream {
    model.elements.values().map(|element_def| {
        let ElementDefinition { ident, .. } = element_def;
        quote! {
            dioxus_core::TemplateNode::Element {
                tag: "#ident",
                namespace: Some("bevy_ui"),
                attrs,
                children,
            } => {
                let children = children
                        .iter()
                        .map(|template_node| Self::from_dioxus(template_node))
                        .collect();
                let attributes = attrs.iter().filter_map(|template_attr| {
                    match template_attr {
                        dioxus_core::nodes::TemplateAttribute::Dynamic { id } => Some(DynamicAttributeId(*id)),
                        dioxus_core::nodes::TemplateAttribute::Static { name, value, namespace } => {
                            warn!("Received static attribute '{name}' with value '{value:#?}' in namespace '{namespace:#?} on element 'node'.");
                            None
                        }
                    }
                }).collect();

                Self::#ident { attributes, children }
            }
        }
    }).collect()
}

fn generate_template_node_spawn(model: &Model) -> proc_macro2::TokenStream {
    let tokens: proc_macro2::TokenStream = model.elements.values().map(|element| {
        let ElementDefinition { ident, components, attributes: _ } = element;

        let insert_components: proc_macro2::TokenStream = components.iter().map(|component_ident| {
            quote! { #component_ident::default() }
        }).collect();

        quote! {
            BevyTemplateNode::(#ident) { children, attributes } => {
                let children = children
                    .iter()
                    .map(|child| child.spawn(world))
                    .collect::<Box<[_]>>();

                let mut entity_builder = world.spawn(#insert_components);
                entity_builder.push_children(&children);
                entity_builder.id()
            },
        } 
    }).collect();

    tokens
}

pub fn generate_mutations(model: &Model) -> syn::Result<proc_macro2::TokenStream> {
    let mutation_applier_tokens = generate_mutation_applier(model)?;
    let template_node_enum_tokens = generate_template_node_enum(model);
    let template_node_from_dioxus = generate_template_node_from_dioxus(model);
    let template_node_spawn = generate_template_node_spawn(model);

    let tokens = quote! {
        #mutation_applier_tokens

        struct DynamicAttributeId(usize);

        #template_node_enum_tokens

        impl BevyTemplateNode {
            fn from_dioxus(node: &dioxus_core::TemplateNode) -> Self {
                match node {
                    #template_node_from_dioxus

                    dioxus_core::TemplateNode::Text { text } => {
                        todo!("TemplateNode::Text");
                        // Self::IntrinsicTextNode(Text::from_section(*text, TextStyle::default()))
                    }
                    dioxus_core::TemplateNode::Dynamic { id: _ } => {
                        todo!("TemplateNode::Dynamic");
                        // Self::Node {
                        //     style: StyleComponents::default(),
                        //     children: Box::new([]),
                        // }
                    },
                    dioxus_core::TemplateNode::DynamicText { id: _ } => {
                        todo!("TemplateNode::DynamicText");
                        // Self::IntrinsicTextNode(Text::from_section("", TextStyle::default()))
                    }
                    dioxus_core::TemplateNode::Element {
                        tag,
                        namespace: None,
                        ..
                    } => {
                        panic!("Encountered unsupported bevy_dioxus tag `{tag}`.")
                    }
                    dioxus_core::TemplateNode::Element {
                        tag,
                        namespace: Some(namespace),
                        ..
                    } => {
                        panic!("Encountered unsupported bevy_dioxus tag `{namespace}::{tag}`.")
                    }
                }
            }
            fn spawn(&self, world: &mut World) -> Entity {
                match self {
                    #template_node_spawn
                }
            }
        }
    };

    Ok(tokens)
}
