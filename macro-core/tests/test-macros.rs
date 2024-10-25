use bevy::prelude::*;
use dioxus_bevy_macro_core::define_bevy_dioxus;
#[test]
fn defines_a_basic_renderer() {
    let renderer = define_bevy_dioxus! {
        @define_prop(position_x, f32, |entity_mut, value| entity_mut.get_mut::<Transform>().unwrap().translation.x = value);
        @define_prop(position_y, f32, |entity_mut, value| entity_mut.get_mut::<Transform>().unwrap().translation.y = value);

        node {
            Transform,

            position_x,
            position_y,
        }
    };
}
