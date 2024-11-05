mod adapter;
mod deferred_system;
mod ecs_hooks;
// mod elements;
mod mutations;
mod tick;

use std::marker::PhantomData;

use bevy_app::{App, Last, Plugin};
use bevy_derive::Deref;
use bevy_ecs::{entity::EntityHashMap, prelude::*};
use bevy_utils::HashMap;
use dioxus::{
    dioxus_core::{ElementId, VirtualDom},
    prelude::Element,
};

use adapter::DioxusBevyTemplateNode;
use deferred_system::DeferredSystemRunQueue;
use ecs_hooks::EcsSubscriptions;
use mutations::BevyTemplate;
use tick::tick_dioxus_ui;

#[derive(Debug, Clone, Copy)]
pub struct DioxusBevyPlugin<TT: DioxusBevyTemplateNode> {
    template_pd: PhantomData<TT>,
}

impl<TT: DioxusBevyTemplateNode> Default for DioxusBevyPlugin<TT> {
    fn default() -> Self {
        Self {
            template_pd: PhantomData
        }
    }
}

impl<TT: DioxusBevyTemplateNode> Plugin for DioxusBevyPlugin<TT> {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<DioxusBevyContext<TT>>()
            .init_resource::<DeferredSystemRunQueue>()
            .add_systems(Last, tick_dioxus_ui::<TT>);
    }
}

struct DioxusBevyContext<TT: DioxusBevyTemplateNode> {
    roots: HashMap<(Entity, DioxusBevyRootComponent), DioxusBevyRoot<TT>>,
    subscriptions: EcsSubscriptions,
}

impl<TT: DioxusBevyTemplateNode> FromWorld for DioxusBevyContext<TT> {
    fn from_world(_world: &mut World) -> Self {
        Self {
            roots: HashMap::default(),
            subscriptions: EcsSubscriptions::default(),
        }
    }
}

#[derive(Component, Deref, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DioxusBevyRootComponent(pub fn() -> Element);

pub struct DioxusBevyRoot<TT: DioxusBevyTemplateNode> {
    virtual_dom: VirtualDom,
    el_to_entity: HashMap<ElementId, Entity>,
    entity_to_el: EntityHashMap<ElementId>,
    templates: HashMap<String, BevyTemplate<TT>>,
    needs_rebuild: bool,
}

impl<TT: DioxusBevyTemplateNode> DioxusBevyRoot<TT> {
    fn new(root_component: DioxusBevyRootComponent) -> Self {
        Self {
            virtual_dom: VirtualDom::new(root_component.0),
            el_to_entity: HashMap::new(),
            entity_to_el: EntityHashMap::default(),
            templates: HashMap::new(),
            needs_rebuild: true,
        }
    }
}

pub mod prelude {
    pub use super::*;
    pub use dioxus;
    pub use dioxus::prelude::{Event as DioxusEvent, *};
    pub use ecs_hooks::*;
    pub use adapter::*;
}
