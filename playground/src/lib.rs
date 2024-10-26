mod deferred_system;
mod ecs_hooks;
mod elements;
mod mutations;
mod tick;

use bevy::{
    app::{Last, Plugin},
    ecs::{entity::EntityHashMap, prelude::*},
    prelude::Deref,
    utils::HashMap,
};
use deferred_system::DeferredSystemRunQueue;
use dioxus::{
    dioxus_core::{ElementId, VirtualDom},
    prelude::Element,
};
use ecs_hooks::EcsSubscriptions;
use mutations::BevyTemplate;
use tick::tick_dioxus_ui;

pub struct DioxusBevyPlugin;
impl Plugin for DioxusBevyPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_non_send_resource::<DioxusBevyContext>()
            .init_resource::<DeferredSystemRunQueue>()
            .add_systems(Last, tick_dioxus_ui);
    }
}

#[derive(Default)]
struct DioxusBevyContext {
    roots: HashMap<(Entity, DioxusBevyRootComponent), DioxusBevyRoot>,
    subscriptions: EcsSubscriptions,
}

#[derive(Component, Deref, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DioxusBevyRootComponent(pub fn() -> Element);

pub struct DioxusBevyRoot {
    virtual_dom: VirtualDom,
    el_to_entity: HashMap<ElementId, Entity>,
    entity_to_el: EntityHashMap<ElementId>,
    templates: HashMap<String, BevyTemplate>,
    needs_rebuild: bool,
}

impl DioxusBevyRoot {
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
    pub use super::elements::*;
    pub use super::*;
    pub use ecs_hooks::*;
    pub use dioxus;
    pub use dioxus::prelude::{Event as DioxusEvent, *};
}
