
pub mod renderer {
    use dioxus_bevy_macro2::define_bevy_dioxus;

    define_bevy_dioxus! {
        #[define_attr]
        pub fn position_x(entity_mut: &mut EntityMut, value: f64) {
            entity_mut.get_mut::<Transform>().unwrap().translation.y = value
        }
        #[define_attr]
        pub fn position_y(entity_mut: &mut EntityMut, value: f64) {
            entity_mut.get_mut::<Transform>().unwrap().translation.y = value
        }

        #[define_element]
        struct node {
            #[component]
            Transform,
        }
    }
}

#[test]
fn defines_a_basic_renderer() {
}
