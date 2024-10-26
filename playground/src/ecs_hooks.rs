use crate::DioxusBevyContext;
use bevy::{
    ecs::{
        component::ComponentId, event::{EventIterator, ManualEventReader}, query::{QueryFilter, ReadOnlyQueryData}, system::{Query, Resource, SystemState}, world::World
    }, prelude::{Event, Events}, utils::{HashMap, HashSet}
};
use dioxus::{
    dioxus_core::{use_hook, ScopeId},
    prelude::{consume_context, current_scope_id, use_drop},
};
use std::any::TypeId;

#[derive(Default)]
pub(crate) struct EcsSubscriptions {
    pub resources: Box<HashMap<ComponentId, HashSet<ScopeId>>>,
    #[allow(clippy::type_complexity)]
    pub events: Box<HashMap<TypeId, (Box<dyn Fn(&World) -> bool>, HashSet<ScopeId>)>>,
    pub world_and_queries: Box<HashSet<ScopeId>>,
}

#[derive(Clone)]
pub(crate) struct EcsContext {
    pub world: *mut World,
}

impl EcsContext {
    pub fn get_world<'a>() -> &'a mut World {
        unsafe { &mut *consume_context::<EcsContext>().world }
    }
}

pub fn use_world<'a>() -> &'a World {
    let world = EcsContext::get_world();

    let scope_id = current_scope_id().unwrap();
    let subscription_manager = use_hook(|| {
        let subscription_manager = &mut world
            .non_send_resource_mut::<DioxusBevyContext>()
            .subscriptions
            .world_and_queries;
        subscription_manager.insert(scope_id);
        Box::as_mut(subscription_manager) as *mut HashSet<ScopeId>
    });
    use_drop(move || {
        unsafe { &mut *subscription_manager }.remove(&scope_id);
    });

    world
}

pub fn use_bevy_resource<'a, T: Resource>() -> &'a T {
    let world = EcsContext::get_world();

    let resource_id = world.components().resource_id::<T>().unwrap();
    let scope_id = current_scope_id().unwrap();
    let subscription_manager = use_hook(|| {
        let subscription_manager = &mut world
            .non_send_resource_mut::<DioxusBevyContext>()
            .subscriptions
            .resources;
        subscription_manager
            .entry(resource_id)
            .or_default()
            .insert(scope_id);
        Box::as_mut(subscription_manager) as *mut HashMap<ComponentId, HashSet<ScopeId>>
    });
    use_drop(move || {
        let subscription_manager = &mut unsafe { &mut *subscription_manager };
        let resource_subscriptions = subscription_manager.get_mut(&resource_id).unwrap();
        resource_subscriptions.remove(&scope_id);
        if resource_subscriptions.is_empty() {
            subscription_manager.remove(&resource_id);
        }
    });

    world.resource()
}

pub fn use_query<'a, Q>() -> UseQuery<'a, Q, ()>
where
    Q: ReadOnlyQueryData,
{
    use_query_filtered()
}

pub fn use_query_filtered<'a, Q, F>() -> UseQuery<'a, Q, F>
where
    Q: ReadOnlyQueryData,
    F: QueryFilter,
{
    let world = EcsContext::get_world();

    let scope_id = current_scope_id().unwrap();
    let subscription_manager = use_hook(|| {
        let subscription_manager = &mut world
            .non_send_resource_mut::<DioxusBevyContext>()
            .subscriptions
            .world_and_queries;
        subscription_manager.insert(scope_id);
        Box::as_mut(subscription_manager) as *mut HashSet<ScopeId>
    });
    use_drop(move || {
        unsafe { &mut *subscription_manager }.remove(&scope_id);
    });

    UseQuery {
        system_state: SystemState::new(world),
        world_ref: world,
    }
}

pub struct UseQuery<'a, Q: ReadOnlyQueryData + 'static, F: QueryFilter + 'static> {
    system_state: SystemState<Query<'static, 'static, Q, F>>,
    world_ref: &'a World,
}

impl<'a, Q, F> UseQuery<'a, Q, F>
where
    Q: ReadOnlyQueryData,
    F: QueryFilter,
{
    pub fn query(&mut self) -> Query<Q, F> {
        self.system_state.get(self.world_ref)
    }
}