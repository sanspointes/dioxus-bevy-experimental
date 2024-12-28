use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_spts_dioxus::*;

// You're going to have to define all of your elements / attributes in here.
#[bevy_spts_dioxus::bevy_spts_dioxus]
pub mod my_adapter {
    use bevy::prelude::*;
    use bevy_spts_dioxus::*;
    use dioxus_core::AttributeValue;

    #[define_attr]
    pub fn name_attr(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        if let Some(name) = value.as_string() {
            entity_mut.insert(Name::from(name.as_str()));
        } else if let Some(name) = value.as_concrete::<Name>() {
            entity_mut.insert(name.clone());
        } else {
            entity_mut.remove::<Name>();
        }
    }

    // Implement custom attributes
    #[define_attr()]
    pub fn is_visible_attr(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        let is_visible = value.as_bool().unwrap_or(false);
        *entity_mut.get_mut::<Visibility>().unwrap() = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    pub mod dioxus_elements {
        use bevy_spts_dioxus::SptsDioxusElement;

        #[define_element]
        pub struct spatial {
            // Components will be spawned with the element.
            #[component]
            transform: Transform,
            #[component]
            visibility: Visibility,

            #[attr]
            name: name_attr,
            // Can compose your attributes across multiple elements.
            #[attr]
            is_visible: is_visible_attr,
        }
        impl SptsDioxusElement for spatial {}
    }
}

use my_adapter::*;

#[component]
pub fn root_component() -> Element {
    let outer_entity = Hooks::use_entity();
    let inner_entity = Hooks::use_entity();

    use_effect(move || {
        println!("outer_entity: {outer_entity:?}, inner_entity: {inner_entity:?}.");
    });
    rsx! {
        spatial {
            entity: outer_entity,
            name: "Outer",
            // Pass your values to your attributes
            is_visible: true,
            // Reactively set whole attributes (must be wrapped with the WA, WrappedAttribute, struct)
            transform: WA(Transform::from_xyz(0., 5., 0.5)),

            // Only dependency is bevy_hierarchy
            spatial {
                entity: inner_entity,
                name: "Inner",
                visibility: WA(Visibility::Visible),
            }
        }
    }
}

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        SptsDioxusRootComponent(root_component),
    ));
}

pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(SptsDioxusPlugin::<my_adapter::SptsDioxusAdapter>::default());
    app.add_plugins(WorldInspectorPlugin::new());
    // Spawn your root bundle.
    app.add_systems(Startup, spawn_root);
    app.run();
}
