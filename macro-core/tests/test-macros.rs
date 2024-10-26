
pub mod renderer {
    use bevy::prelude::*;
    use dioxus_bevy_macro_core::define_bevy_dioxus;

    define_bevy_dioxus! {
        @define_prop(position_x, f64, |entity_mut, value| entity_mut.get_mut::<Transform>().unwrap().translation.x = value);
        @define_prop(position_y, f64, |entity_mut, value| );

        #[attr]
        pub fn position_x(entity_mut: &mut EntityMut, value: f64) {
            entity_mut.get_mut::<Transform>().unwrap().translation.y = value
        }
        pub fn position_y(entity_mut: &mut EntityMut, value: f64) {
            entity_mut.get_mut::<Transform>().unwrap().translation.y = value
        }

        node {
            Transform,

            position_x,
            position_y,
        }
    }
}

#[test]
fn defines_a_basic_renderer() {
}
