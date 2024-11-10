//! Examples: no_macro
//!
//! This showcases how to use this library without using the macro
//! to generate the rendering adapter for your project.
//!
//! It works by manually defining the dioxus_elements module + the elements enum + implementing the
//! DioxusBevyTemplateNode trait on the elements enum to provide all the logic to spawn and mutate
//! the elements.  It's a bit of a pain because a few things need to be kept in sync.
//!
//! 1. If you define an attribute in dioxus_elements you must handle it in `set_attribute`.
//! 2. If you define an element in `dioxus_elements` you must handle it in `from_dioxus` and `spawn`
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use dioxus_bevy::*;
use my_adapter::*;

#[allow(non_camel_case_types)]
#[dioxus_bevy]
mod my_adapter {
    use bevy::prelude::*;
    use dioxus_bevy::*;
    use dioxus_core::AttributeValue;

    #[define_attr]
    pub fn position(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        entity_mut.get_mut::<Transform>().unwrap().translation =
            value.as_concrete::<Vec3>().copied().unwrap_or_default()
    }
    #[define_attr]
    pub fn position_x(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        entity_mut.get_mut::<Transform>().unwrap().translation.x = value.as_f32().unwrap_or(0.)
    }
    #[define_attr]
    pub fn position_y(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        entity_mut.get_mut::<Transform>().unwrap().translation.y = value.as_f32().unwrap_or(0.)
    }
    #[define_attr]
    pub fn position_z(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        entity_mut.get_mut::<Transform>().unwrap().translation.z = value.as_f32().unwrap_or(0.)
    }

    #[define_attr]
    pub fn mesh_handle(world: &mut World, entity: Entity, value: &AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        if let Some(mesh_handle) = value.as_concrete::<Handle<Mesh>>() {
            entity_mut.insert(mesh_handle.clone());
        } else {
            entity_mut.remove::<Handle<Mesh>>();
        }
    }


    pub mod dioxus_elements {
        use dioxus_bevy::DioxusBevyElement;

        #[define_element]
        struct spatial {
            #[component]
            transform: Transform,
            #[component]
            global_transform: GlobalTransform,
            #[component]
            visibility: Visibility,
            #[component]
            inherited_visibility: InheritedVisibility,
            #[component]
            view_visibility: ViewVisibility,

            #[attr]
            position: position,
            #[attr]
            position_x: position_x,
            #[attr]
            position_y: position_y,
            #[attr]
            position_z: position_z,
        }
        impl DioxusBevyElement for spatial {}

        #[define_element]
        struct mesh {
            #[component]
            transform: Transform,
            #[component]
            global_transform: GlobalTransform,
            #[component]
            visibility: Visibility,
            #[component]
            inherited_visibility: InheritedVisibility,
            #[component]
            view_visibility: ViewVisibility,

            #[attr]
            mesh_handle: mesh_handle,

            #[attr]
            position: position,
            #[attr]
            position_x: position_x,
            #[attr]
            position_y: position_y,
            #[attr]
            position_z: position_z,
        }
        impl DioxusBevyElement for mesh {}
    }
}

#[derive(Resource)]
pub struct State {
    pressed_count: usize,
}
pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DioxusBevyPlugin::<my_adapter::DioxusBevyAdapter>::default());
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_systems(Startup, setup);

    app.insert_resource(State { pressed_count: 0 });
    app.add_systems(Update, update);

    app.run();
}

#[component]
pub fn root() -> Element {
    println!("Re-rendering root node");

    let state = DBHooks::<my_adapter::DioxusBevyAdapter>::use_bevy_resource::<State>();

    rsx! {
        spatial {
            position_x: state.pressed_count as f64,
            position_y: 1.0,
            position_z: 0.5,
            visibility: WA(if state.pressed_count % 2 == 0 { Visibility::Visible } else { Visibility::Hidden }),
            spatial {
                position: WA(Vec3::new(state.pressed_count as f32, 0., 0.)),
            }
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        DioxusBevyRootComponent(root),
    ));
}

pub fn update(button_state: Res<ButtonInput<KeyCode>>, mut state: ResMut<State>) {
    if button_state.pressed(KeyCode::Space) {
        state.pressed_count += 1
    }
}
