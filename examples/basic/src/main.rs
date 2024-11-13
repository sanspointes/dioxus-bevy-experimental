use bevy::prelude::*;
use dioxus_bevy::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// You're going to have to define all of your elements / attributes in here.
#[dioxus_bevy::dioxus_bevy]
pub mod my_adapter {
    use bevy::prelude::*;
    use dioxus_bevy::*;
    use dioxus_core::AttributeValue;


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
        use dioxus_bevy::DioxusBevyElement;

        #[define_element]
        pub struct spatial {
            // Components will be spawned with the element.
            #[component]
            transform: Transform,
            #[component]
            visibility: Visibility,

            // Can compose your attributes across multiple elements. 
            #[attr]
            is_visible: is_visible_attr,
        }
        impl DioxusBevyElement for spatial {}

    }
}

use my_adapter::*;

#[component]
pub fn root_component() -> Element {
    rsx! {
        spatial {
            // Pass your values to your attributes
            is_visible: true,
            // Reactively set whole attributes (must be wrapped with the WA, WrappedAttribute, struct)
            transform: WA(Transform::from_xyz(0., 5., 0.5)),

            // Only dependency is bevy_hierarchy
            spatial {
                visibility: WA(Visibility::Visible),
            }
        }
    }
}

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), DioxusBevyRootComponent(root_component)));
}

pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DioxusBevyPlugin::<my_adapter::DioxusBevyAdapter>::default());
    app.add_plugins(WorldInspectorPlugin::new());
    // Spawn your root bundle.
    app.add_systems(Startup, spawn_root);
    app.run();
}

