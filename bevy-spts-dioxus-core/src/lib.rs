mod adapter;
mod deferred_system;
mod ecs_hooks;
// mod elements;
mod mutations;
mod tick;
mod hooks;
mod utils;

use std::marker::PhantomData;

use bevy_app::{App, Last, Plugin};
use bevy_derive::Deref;
use bevy_ecs::{entity::EntityHashMap, prelude::*};
use bevy_utils::HashMap;
use dioxus::{
    dioxus_core::{ElementId, VirtualDom},
    prelude::Element, signals::Signal,
};

use adapter::SptsDioxusTemplateNode;
use deferred_system::DeferredSystemRunQueue;
use ecs_hooks::EcsSubscriptions;
use mutations::BevyTemplate;
use tick::tick_dioxus_ui;

#[derive(Debug, Clone, Copy)]
pub struct SptsDioxusPlugin<TT: SptsDioxusTemplateNode> {
    template_pd: PhantomData<TT>,
}

impl<TT: SptsDioxusTemplateNode> Default for SptsDioxusPlugin<TT> {
    fn default() -> Self {
        Self {
            template_pd: PhantomData,
        }
    }
}

impl<TT: SptsDioxusTemplateNode> Plugin for SptsDioxusPlugin<TT> {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<SptsDioxusContext<TT>>()
            .init_resource::<DeferredSystemRunQueue>()
            .add_systems(Last, tick_dioxus_ui::<TT>);
    }
}

pub struct SptsDioxusContext<TT: SptsDioxusTemplateNode> {
    roots: HashMap<(Entity, SptsDioxusRootComponent), SptsDioxusRoot<TT>>,
    subscriptions: EcsSubscriptions,
}

impl<TT: SptsDioxusTemplateNode> FromWorld for SptsDioxusContext<TT> {
    fn from_world(_world: &mut World) -> Self {
        Self {
            roots: HashMap::default(),
            subscriptions: EcsSubscriptions::default(),
        }
    }
}

#[derive(Component, Deref, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SptsDioxusRootComponent(pub fn() -> Element);

pub struct SptsDioxusRoot<TT: SptsDioxusTemplateNode> {
    virtual_dom: VirtualDom,
    el_to_entity: HashMap<ElementId, Entity>,
    entity_to_el: EntityHashMap<ElementId>,
    entity_refs: EntityHashMap<Signal<Option<Entity>>>,
    templates: HashMap<String, BevyTemplate<TT>>,
    needs_rebuild: bool,
}

impl<TT: SptsDioxusTemplateNode> SptsDioxusRoot<TT> {
    fn new(root_component: SptsDioxusRootComponent) -> Self {
        Self {
            virtual_dom: VirtualDom::new(root_component.0),
            el_to_entity: HashMap::new(),
            entity_to_el: EntityHashMap::default(),
            entity_refs: EntityHashMap::default(),
            templates: HashMap::new(),
            needs_rebuild: true,
        }
    }
}

pub mod prelude {
    pub use super::{SptsDioxusContext, SptsDioxusPlugin, SptsDioxusRoot, SptsDioxusRootComponent};
    pub use crate::adapter::*;
    pub use crate::ecs_hooks::*;
    pub use crate::utils::*;
    pub use dioxus;
    pub use dioxus::prelude::{Event as DioxusEvent, *};
}
