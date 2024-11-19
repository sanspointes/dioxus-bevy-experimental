use crate::{adapter::SptsDioxusTemplateNode, SptsDioxusContext};
use bevy_ecs::{
    component::ComponentId,
    query::{QueryFilter, ReadOnlyQueryData},
    system::{Query, Resource, SystemState},
    world::World,
};
use bevy_utils::{HashMap, HashSet};
use dioxus::{
    dioxus_core::{use_hook, ScopeId},
    hooks::{use_callback, use_memo, UseCallback},
    prelude::{consume_context, current_scope_id, use_drop},
    signals::Memo,
};
use std::{any::TypeId, marker::PhantomData};

#[derive(Default)]
pub(crate) struct EcsSubscriptions {
    pub resources: Box<HashMap<ComponentId, HashSet<ScopeId>>>,
    #[allow(clippy::type_complexity)]
    pub events: Box<HashMap<TypeId, (Box<dyn Fn(&World) -> bool>, HashSet<ScopeId>)>>,
    pub world_and_queries: Box<HashSet<ScopeId>>,
}

/// Struct that has static functions for hooks that use the correct adapter.
///
/// * `pd`:
pub struct SptsDioxusHooks<TT: SptsDioxusTemplateNode> {
    pd: PhantomData<TT>,
}

#[derive(Clone)]
pub(crate) struct EcsContext<TT: SptsDioxusTemplateNode> {
    pub world: *mut World,
    pd: PhantomData<TT>,
}

impl<TT: SptsDioxusTemplateNode> EcsContext<TT> {
    pub fn new(world: *mut World) -> Self {
        Self {
            world,
            pd: PhantomData,
        }
    }
}

impl<TT: SptsDioxusTemplateNode> EcsContext<TT> {
    pub fn get_world<'a>() -> &'a mut World {
        unsafe { &mut *consume_context::<EcsContext<TT>>().world }
    }
}

impl<TT: SptsDioxusTemplateNode> SptsDioxusHooks<TT> {
    pub fn use_world<'a>() -> &'a World {
        let world = EcsContext::<TT>::get_world();

        let scope_id = current_scope_id().unwrap();
        let subscription_manager = use_hook(|| {
            let subscription_manager = &mut world
                .non_send_resource_mut::<SptsDioxusContext<TT>>()
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
        let world = EcsContext::<TT>::get_world();

        let resource_id = world.components().resource_id::<T>().unwrap();
        let scope_id = current_scope_id().unwrap();
        let subscription_manager = use_hook(|| {
            let subscription_manager = &mut world
                .non_send_resource_mut::<SptsDioxusContext<TT>>()
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
}

impl<TT: SptsDioxusTemplateNode> SptsDioxusHooks<TT> {
    pub fn use_query<'a, Q>() -> UseQuery<'a, Q, ()>
    where
        Q: ReadOnlyQueryData,
    {
        Self::use_query_filtered()
    }

    pub fn use_query_filtered<'a, Q, F>() -> UseQuery<'a, Q, F>
    where
        Q: ReadOnlyQueryData,
        F: QueryFilter,
    {
        let world = EcsContext::<TT>::get_world();

        let scope_id = current_scope_id().unwrap();
        let subscription_manager = use_hook(|| {
            let subscription_manager = &mut world
                .non_send_resource_mut::<SptsDioxusContext<TT>>()
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

impl<TT: SptsDioxusTemplateNode> SptsDioxusHooks<TT> {
    pub fn use_world_memo<TResult: PartialEq>(
        mut memo_fn: impl FnMut(&mut World) -> TResult + 'static,
    ) -> Memo<TResult> {
        let value = use_memo(move || {
            let world = EcsContext::<TT>::get_world();
            memo_fn(world)
        });
        value
    }

    pub fn use_world_callback<TResult>(
        mut callback_fn: impl FnMut(&mut World) -> TResult + 'static,
    ) -> UseCallback<TResult> {
        use_callback(move || {
            let world = EcsContext::<TT>::get_world();
            callback_fn(world)
        })
    }
}
