use bevy::prelude::Component;
use dioxus::{dioxus_core::AttributeValue, prelude::IntoAttributeValue};


#[allow(non_camel_case_types, non_upper_case_globals)]
pub mod dioxus_bevy_attributes {
    use bevy::prelude::{EntityWorldMut, Transform};

    pub fn position_x(mut entity_mut: EntityWorldMut, value: f64) {
        entity_mut.get_mut::<Transform>().unwrap().translation.x = value as f32
    }
}

pub mod dioxus_elements {
    pub type AttributeDescription = (&'static str, Option<&'static str>, bool);
    const NAME_SPACE: Option<&'static str> = Some("bevy_ui");

    #[allow(non_camel_case_types)]
    pub struct node; 
    #[allow(non_upper_case_globals)]
    impl node {
        pub const TAG_NAME: &'static str = "node";
        pub const NAME_SPACE: Option<&'static str> = NAME_SPACE;
        pub const position_x: AttributeDescription = ("position_x", None, false);
        pub const transform: AttributeDescription = ("transform", None, false);
        pub const visibility: AttributeDescription = ("visibility", None, false);
    }
}

pub struct C<T: Component + PartialEq>(pub T);

impl<T: Component + PartialEq> IntoAttributeValue for C<T> {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Any(Box::new(self.0))
    }
}

pub struct OC<T: Component + PartialEq>(pub Option<T>);

impl<T: Component + PartialEq> IntoAttributeValue for OC<T> {
    fn into_value(self) -> AttributeValue {
        match self.0 {
            Some(v) => AttributeValue::Any(Box::new(v)),
            None => AttributeValue::None,
        }
    }
}
