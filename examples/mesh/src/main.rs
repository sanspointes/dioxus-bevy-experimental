//! Examples: no_macro
//!
//! This showcases how to use this library without using the macro
//! to generate the rendering adapter for your project.
//!
//! It works by manually defining the dioxus_elements module + the elements enum + implementing the
//! SptsDioxusTemplateNode trait on the elements enum to provide all the logic to spawn and mutate
//! the elements.  It's a bit of a pain because a few things need to be kept in sync.
//!
//! 1. If you define an attribute in dioxus_elements you must handle it in `set_attribute`.
//! 2. If you define an element in `dioxus_elements` you must handle it in `from_dioxus` and `spawn`
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_spts_dioxus::*;
use my_adapter::*;

#[allow(non_camel_case_types)]
#[bevy_spts_dioxus]
mod my_adapter {
    use bevy::prelude::*;
    use bevy_spts_dioxus::*;
    use dioxus_core::AttributeValue;

    use bevy::{
        core_pipeline::tonemapping::Tonemapping,
        render::{
            camera::{CameraMainTextureUsages, Exposure},
            primitives::Frustum,
            view::{ColorGrading, VisibleEntities},
        },
    };

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
            println!("Setting mesh to {mesh_handle:?}");
            entity_mut.insert(mesh_handle.clone());
        } else {
            entity_mut.remove::<Handle<Mesh>>();
        }
    }

    #[define_attr]
    pub fn color(world: &mut World, entity: Entity, value: &AttributeValue) {
        world.resource_scope::<Assets<StandardMaterial>, ()>(|world, mut color_materials| {
            if let Some(handle) = world
                .entity_mut(entity)
                .get::<Handle<StandardMaterial>>()
                .cloned()
            {
                color_materials.remove(&handle);
            }

            let value = *value.as_concrete::<Color>().unwrap_or_else(|| {
                panic!("bevy_spts_dioxus: 'color' attribute error unwrapping 'Color'.  Found {value:?}.")
            });
            let handle = color_materials.add(value);
            world.entity_mut(entity).insert(handle);
        });
    }

    pub mod dioxus_elements {
        use bevy::{
            core_pipeline::{core_3d::graph::Core3d, tonemapping::DebandDither},
            render::camera::CameraRenderGraph,
        };
        use bevy_spts_dioxus::SptsDioxusElement;

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
        impl SptsDioxusElement for spatial {}

        #[define_element]
        struct colormesh {
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

            #[attr]
            mesh_handle: mesh_handle,

            #[attr]
            color: color,
        }
        impl SptsDioxusElement for colormesh {}

        #[define_element]
        struct perspectivecamera {
            #[component]
            transform: Transform,
            #[component]
            global_transform: GlobalTransform,

            #[component]
            pub camera: Camera,
            #[component]
            pub projection: Projection,
            #[component]
            pub visible_entities: VisibleEntities,
            #[component]
            pub frustum: Frustum,
            #[component]
            pub camera_3d: Camera3d,
            #[component]
            pub tonemapping: Tonemapping,
            #[component]
            pub color_grading: ColorGrading,
            #[component]
            pub exposure: Exposure,
            #[component]
            pub main_texture_usages: CameraMainTextureUsages,

            #[attr]
            position: position,
            #[attr]
            position_x: position_x,
            #[attr]
            position_y: position_y,
            #[attr]
            position_z: position_z,
        }
        impl SptsDioxusElement for perspectivecamera {
            fn spawn(world: &mut bevy::ecs::world::World) -> bevy::prelude::EntityWorldMut<'_> {
                world.spawn((CameraRenderGraph::new(Core3d), DebandDither::Enabled))
            }
        }
    }
}

#[derive(Resource)]
pub struct State {
    pressed_count: usize,
}
pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(SptsDioxusPlugin::<my_adapter::SptsDioxusAdapter>::default());
    app.add_plugins(WorldInspectorPlugin::new());

    app.add_systems(Startup, setup);

    app.insert_resource(State { pressed_count: 0 });
    app.add_systems(Update, update);

    app.run();
}

#[component]
pub fn root() -> Element {
    let state = Hooks::use_bevy_resource::<State>();

    let mesh = Hooks::use_world_memo(|world| {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        meshes.add(Mesh::from(Sphere::new(0.1)))
    });

    rsx! {
        perspectivecamera {
            position_z: 12,
        }

        spatial {
            transform: WA(Transform::from_xyz(0., 5., 0.)),
            position_x: (state.pressed_count as f64) * 0.01,
            position_y: 1.0,
            position_z: 0.5,
            visibility: WA(if state.pressed_count % 2 == 0 { Visibility::Visible } else { Visibility::Hidden }),

            for i in 0..16 {
                colormesh {
                    position_x: f64::from(i).sin(),
                    position_y: f64::from(i).cos(),
                    position_z: f64::from(i).cos(),
                    color: WA(Color::srgb(1., 0., 0.)),
                    mesh_handle: WA(mesh.read().clone_weak()),
                }
            }
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), SptsDioxusRootComponent(root)));
}

pub fn update(button_state: Res<ButtonInput<KeyCode>>, mut state: ResMut<State>) {
    if button_state.pressed(KeyCode::Space) {
        state.pressed_count += 1
    }
}
