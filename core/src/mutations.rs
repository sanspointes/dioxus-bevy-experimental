use bevy_ecs::{
    entity::{Entity, EntityHashMap},
    system::{Query, SystemState},
    world::{Command, World},
};
use bevy_hierarchy::{BuildWorldChildren, Children, DespawnRecursive, HierarchyQueryExt, Parent};
use bevy_utils::HashMap;
use dioxus::{
    dioxus_core::{AttributeValue, ElementId, WriteMutations},
    prelude::Template,
};

use crate::adapter::DioxusBevyTemplateNode;

pub struct MutationApplier<'a, TT: DioxusBevyTemplateNode> {
    el_to_entity: &'a mut HashMap<ElementId, Entity>,
    entity_to_el: &'a mut EntityHashMap<ElementId>,
    templates: &'a mut HashMap<String, BevyTemplate<TT>>,
    world: &'a mut World,
    stack: Vec<Entity>,
}

impl<'a, TT: DioxusBevyTemplateNode> MutationApplier<'a, TT> {
    pub fn new(
        el_to_entity: &'a mut HashMap<ElementId, Entity>,
        entity_to_el: &'a mut EntityHashMap<ElementId>,
        templates: &'a mut HashMap<String, BevyTemplate<TT>>,
        root_entity: Entity,
        world: &'a mut World,
    ) -> Self {
        el_to_entity.insert(ElementId(0), root_entity);
        entity_to_el.insert(root_entity, ElementId(0));

        Self {
            el_to_entity,
            entity_to_el,
            templates,
            world,
            stack: vec![root_entity],
        }
    }
}

impl<'a, TT: DioxusBevyTemplateNode> MutationApplier<'a, TT> {
    pub fn despawn_recursive(&mut self, entity: Entity) {
        let mut ss: SystemState<Query<&Children>> = SystemState::new(self.world);
        let query_children = ss.get_mut(self.world);
        for child in query_children.iter_descendants(entity) {
            if let Some(existing_element_id) = self.entity_to_el.remove(&child) {
                self.el_to_entity.remove(&existing_element_id);
            }
        }

        if let Some(existing_element_id) = self.entity_to_el.remove(&entity) {
            self.el_to_entity.remove(&existing_element_id);
        }

        DespawnRecursive { entity }.apply(self.world);
    }
}

impl<'a, TT: DioxusBevyTemplateNode> WriteMutations for MutationApplier<'a, TT> {
    fn register_template(&mut self, template: Template) {
        self.templates.insert(
            template.name.to_owned(),
            BevyTemplate::from_dioxus(&template),
        );
    }

    fn append_children(&mut self, id: ElementId, m: usize) {
        let mut parent = self.world.entity_mut(self.el_to_entity[&id]);
        for child in self.stack.drain((self.stack.len() - m)..) {
            parent.add_child(child);
        }
    }

    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        let mut entity = *self.stack.last().unwrap();
        for index in path {
            entity = self.world.entity(entity).get::<Children>().unwrap()[*index as usize];
        }
        self.el_to_entity.insert(id, entity);
        self.entity_to_el.insert(entity, id);
    }

    fn create_placeholder(&mut self, id: ElementId) {
        let entity = self.world.spawn_empty().id();
        self.el_to_entity.insert(id, entity);
        self.entity_to_el.insert(entity, id);
        self.stack.push(entity);
    }

    fn create_text_node(&mut self, value: &str, id: ElementId) {
        todo!("create_text_node");
        // let entity =
        //     BevyTemplateNode::IntrinsicTextNode(Text::from_section(value, TextStyle::default()))
        //         .spawn(self.world);
        // self.el_to_entity.insert(id, entity);
        // self.entity_to_el.insert(entity, id);
        // self.stack.push(entity);
    }

    fn hydrate_text_node(&mut self, path: &'static [u8], value: &str, id: ElementId) {
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

    fn load_template(&mut self, name: &'static str, index: usize, id: ElementId) {
        let entity = self.templates[name].roots[index].spawn(self.world);
        self.el_to_entity.insert(id, entity);
        self.entity_to_el.insert(entity, id);
        self.stack.push(entity);
    }

    fn replace_node_with(&mut self, id: ElementId, m: usize) {
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

    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
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

    fn insert_nodes_before(&mut self, id: ElementId, m: usize) {
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

    fn set_attribute(
        &mut self,
        name: &'static str,
        ns: Option<&'static str>,
        value: &AttributeValue,
        id: ElementId,
    ) {
        let entity = self.el_to_entity[&id];
        let entity_mut = self.world.entity_mut(entity);

        println!("Setting attribute '{name}' to '{value:?}");

        TT::apply_attribute(entity_mut, name, value);
        // match name {
        //     "position_x" => {
        //         match value {
        //             AttributeValue::Float(value) =>
        //                     dioxus_bevy_attributes::position_x(entity_mut, *value),
        //             AttributeValue::None => todo!("Handle removing attribute."),
        //             other => warn!("Incorrect value passed to 'position_x' attribute. Expected 'f64' but found {other:?}."),
        //         }
        //     },
        //     "transform" => self.apply_required_component::<Transform>(entity, value),
        //     "visibility" => self.apply_optional_component::<Visibility>(entity, value),
        //     other => {
        //         let stringified_namespace = ns.unwrap_or("dioxus_bevy");
        //         warn!("{stringified_namespace}: Received unknown attribute {other:?} with value {value:?}");
        //     },
        // }
    }

    fn set_node_text(&mut self, value: &str, id: ElementId) {
        todo!("set_node_text");
    }

    fn create_event_listener(&mut self, name: &'static str, id: ElementId) {
        todo!("create_event_listener");
        // insert_event_listener(
        //     &name,
        //     self.world
        //         .entity_mut(self.el_to_entity[&id]),
        // );
    }

    fn remove_event_listener(&mut self, name: &'static str, id: ElementId) {
        todo!("remove_event_listener");
        // remove_event_listener(
        //     &name,
        //     self.world
        //         .entity_mut(self.el_to_entity[&id]),
        // );
    }

    fn remove_node(&mut self, id: ElementId) {
        let entity = self.el_to_entity[&id];
        self.despawn_recursive(entity);
    }

    fn push_root(&mut self, id: ElementId) {
        self.stack.push(self.el_to_entity[&id]);
    }
}

pub struct BevyTemplate<TT: DioxusBevyTemplateNode> {
    roots: Box<[TT]>,
}

impl<TT: DioxusBevyTemplateNode> BevyTemplate<TT> {
    fn from_dioxus(template: &Template) -> Self {
        Self {
            roots: template
                .roots
                .iter()
                .map(|node| TT::from_dioxus(node))
                .collect(),
        }
    }
}

// pub struct DynamicAttributeId(usize);
//
// pub enum BevyTemplateNode {
//     Node {
//         attributes: Box<[DynamicAttributeId]>,
//         children: Box<[Self]>,
//     }
// }
// impl BevyTemplateNode {
//     fn from_dioxus(node: &TemplateNode) -> Self {
//         match node {
//             TemplateNode::Element {
//                 tag: "node",
//                 namespace: Some("bevy_ui"),
//                 attrs,
//                 children,
//             } => {
//                 let children = children
//                         .iter()
//                         .map(|template_node| Self::from_dioxus(template_node))
//                         .collect();
//                 let attributes = attrs.iter().filter_map(|template_attr| {
//                     match template_attr {
//                         TemplateAttribute::Dynamic { id } => Some(DynamicAttributeId(*id)),
//                         TemplateAttribute::Static { name, value, namespace } => {
//                             warn!("Received static attribute '{name}' with value '{value:#?}' in namespace '{namespace:#?} on element 'node'.");
//                             None
//                         }
//                     }
//                 }).collect();
//
//                 Self::Node { attributes, children }
//             }
//             TemplateNode::Text { text } => {
//                 todo!("TemplateNode::Text");
//                 // Self::IntrinsicTextNode(Text::from_section(*text, TextStyle::default()))
//             }
//             TemplateNode::Dynamic { id: _ } => {
//                 todo!("TemplateNode::Dynamic");
//                 // Self::Node {
//                 //     style: StyleComponents::default(),
//                 //     children: Box::new([]),
//                 // }
//             },
//             TemplateNode::DynamicText { id: _ } => {
//                 todo!("TemplateNode::DynamicText");
//                 // Self::IntrinsicTextNode(Text::from_section("", TextStyle::default()))
//             }
//             TemplateNode::Element {
//                 tag,
//                 namespace: None,
//                 ..
//             } => {
//                 panic!("Encountered unsupported bevy_dioxus tag `{tag}`.")
//             }
//             TemplateNode::Element {
//                 tag,
//                 namespace: Some(namespace),
//                 ..
//             } => {
//                 panic!("Encountered unsupported bevy_dioxus tag `{namespace}::{tag}`.")
//             }
//         }
//     }
//     fn spawn(&self, world: &mut World) -> Entity {
//         match self {
//             BevyTemplateNode::Node { children, attributes } => {
//                 let children = children
//                     .iter()
//                     .map(|child| child.spawn(world))
//                     .collect::<Box<[_]>>();
//
//                 let mut entity_builder = world.spawn_empty();
//                 entity_builder.insert((Transform::default(), GlobalTransform::default()));
//                 entity_builder.push_children(&children);
//                 let entity = entity_builder.id();
//                 println!("Spawned BevyTemplateNode::Node with id {entity:?}");
//                 entity
//             }
//         }
//     }
// }
