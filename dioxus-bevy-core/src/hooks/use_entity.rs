use bevy_derive::{Deref, DerefMut};
use bevy_ecs::entity::Entity;
use dioxus::{
    dioxus_core::AttributeValue,
    hooks::use_signal,
    prelude::{use_hook, IntoAttributeValue},
    signals::Signal,
};

use crate::{adapter::DioxusBevyTemplateNode, prelude::DioxusBevyHooks};

#[derive(Debug, PartialEq, Clone, Copy, Deref, DerefMut)]
pub struct EntitySignal(Signal<Option<Entity>>);

impl IntoAttributeValue for EntitySignal {
    fn into_value(self) -> dioxus::dioxus_core::AttributeValue {
        AttributeValue::any_value(self)
    }
}

impl<TT: DioxusBevyTemplateNode> DioxusBevyHooks<TT> {
    /// Returns an EntitySignal that can be passed via the `entity` attribute
    /// to an element to get a handle to the entity in the scene.
    ///
    /// # Example
    ///
    /// ```
    /// #[component]
    /// pub fn MyComponent() {
    ///     let my_entity = Hooks:use_entity();
    ///     rsx! {
    ///         spatial {
    ///             entity: my_entity,
    ///         }
    ///     }
    /// }
    /// ```
    pub fn use_entity() -> EntitySignal {
        let signal = use_signal::<Option<Entity>>(|| None);
        EntitySignal(signal)
    }
}
