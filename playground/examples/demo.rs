use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use playground::prelude::*;
use playground::{DioxusBevyPlugin, DioxusBevyRootComponent};

#[derive(Resource)]
pub struct State {
    pressed_count: usize,
}
pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DioxusBevyPlugin);
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_systems(Startup, setup);

    app.insert_resource(State { pressed_count: 0 });
    app.add_systems(Update, update);

    app.run();
}

#[component]
pub fn root() -> Element {
    println!("Spawning root node");

    let state = use_bevy_resource::<State>();

    rsx! {
        node {
            transform: C(Transform::default().with_scale(Vec3::new(0.5, 2., 1.))),
            node {
                position_x: state.pressed_count as f64,
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
