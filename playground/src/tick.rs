use bevy::{
    asset::{AssetServer, AssetServerMode}, prelude::{Entity, Mut, World}, utils::HashMap
};

use crate::{ecs_hooks::EcsContext, mutations::MutationApplier, prelude::DeferredSystemRunQueue, DioxusBevyContext, DioxusBevyRoot, DioxusBevyRootComponent};

pub fn tick_dioxus_ui(world: &mut World) {
    run_deferred_systems(world);

    // let ui_events = world.resource_scope(|world, mut event_readers: Mut<EventReaders>| {
    //     event_readers.read_events(
    //         world.resource(),
    //         world.resource(),
    //         world.resource(),
    //         world.resource(),
    //         world.resource(),
    //         world.resource(),
    //         world.resource(),
    //     )
    // });

    let root_entities: HashMap<Entity, DioxusBevyRootComponent> = world
        .query::<(Entity, &DioxusBevyRootComponent)>()
        .iter(world)
        .map(|(entity, root_component)| (entity, *root_component))
        .collect();
    let mut roots =
        std::mem::take(&mut world.non_send_resource_mut::<DioxusBevyContext>().roots);

    for (root_entity, dioxus_ui_root) in root_entities {
        let mut root = roots
            .remove(&(root_entity, dioxus_ui_root))
            .unwrap_or_else(|| DioxusBevyRoot::new(dioxus_ui_root));

        // dispatch_ui_events(&ui_events, &mut ui_root, world);

        schedule_ui_renders_from_ecs_subscriptions(&mut root, world);

        render_ui(root_entity, &mut root, world);

        world
            .non_send_resource_mut::<DioxusBevyContext>()
            .roots
            .insert((root_entity, dioxus_ui_root), root);
    }
}

fn run_deferred_systems(world: &mut World) {
    for mut system in std::mem::take(&mut *world.resource_mut::<DeferredSystemRunQueue>().run_queue)
    {
        system.initialize(world);
        system.run((), world);
    }
}

fn schedule_ui_renders_from_ecs_subscriptions(ui_root: &mut DioxusBevyRoot, world: &World) {
    let ecs_subscriptions = &world.non_send_resource::<DioxusBevyContext>().subscriptions;

    for scope_id in &*ecs_subscriptions.world_and_queries {
        ui_root.virtual_dom.mark_dirty(*scope_id);
    }

    for (resource_id, scope_ids) in &*ecs_subscriptions.resources {
        if world.is_resource_changed_by_id(*resource_id) {
            for scope_id in scope_ids {
                ui_root.virtual_dom.mark_dirty(*scope_id);
            }
        }
    }

    for (new_events_exist, scope_ids) in ecs_subscriptions.events.values() {
        if new_events_exist(world) {
            for scope_id in scope_ids {
                ui_root.virtual_dom.mark_dirty(*scope_id);
            }
        }
    }
}

fn render_ui(root_entity: Entity, ui_root: &mut DioxusBevyRoot, world: &mut World) {
    ui_root
        .virtual_dom
        .provide_root_context(EcsContext { world });

    #[cfg(feature = "hot_reload")]
    crate::hot_reload::update_templates(world, &mut ui_root.virtual_dom);

    if ui_root.needs_rebuild {
        world.resource_scope(|world, _asset_server: Mut<AssetServer>| {
            let mut mutation_applier = MutationApplier::new(
                &mut ui_root.el_to_entity,
                &mut ui_root.entity_to_el,
                &mut ui_root.templates,
                root_entity,
                world,
            );
            ui_root.virtual_dom.rebuild(&mut mutation_applier);
        });
        ui_root.needs_rebuild = false;
    }

    world.resource_scope(|world, _asset_server: Mut<AssetServer>| {
        let mut mutation_applier = MutationApplier::new(
            &mut ui_root.el_to_entity,
            &mut ui_root.entity_to_el,
            &mut ui_root.templates,
            root_entity,
            world,
        );
        ui_root.virtual_dom.render_immediate(&mut mutation_applier);
    });
}
