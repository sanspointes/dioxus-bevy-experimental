mod mutations;
mod dioxus_elements;
mod attr_implementations;

use attr_implementations::generate_attr_implementations;
use dioxus_elements::generate_elements;
use mutations::generate_mutations;
use quote::quote;

use crate::parser::Model;

fn generate_bevy() -> proc_macro2::TokenStream {
    quote! {
        pub struct DioxusBevyPlugin;
        impl Plugin for DioxusBevyPlugin {
            fn build(&self, app: &mut bevy::prelude::App) {
                app.init_non_send_resource::<DioxusBevyContext>()
                    .init_resource::<DeferredSystemRunQueue>()
                    .add_systems(Last, tick_dioxus_ui);
            }
        }

        #[derive(Default)]
        struct DioxusBevyContext {
            roots: HashMap<(Entity, DioxusBevyRootComponent), DioxusBevyRoot>,
            subscriptions: EcsSubscriptions,
        }

        #[derive(Component, Deref, Hash, PartialEq, Eq, Clone, Copy)]
        pub struct DioxusBevyRootComponent(pub fn() -> Element);

        pub struct DioxusBevyRoot {
            virtual_dom: VirtualDom,
            el_to_entity: HashMap<ElementId, Entity>,
            entity_to_el: EntityHashMap<ElementId>,
            templates: HashMap<String, BevyTemplate>,
            needs_rebuild: bool,
        }

        impl DioxusBevyRoot {
            fn new(root_component: DioxusBevyRootComponent) -> Self {
                Self {
                    virtual_dom: VirtualDom::new(root_component.0),
                    el_to_entity: HashMap::new(),
                    entity_to_el: EntityHashMap::default(),
                    templates: HashMap::new(),
                    needs_rebuild: true,
                }
            }
        }

    }
}

pub fn generate(model: Model) -> syn::Result<proc_macro2::TokenStream> {
    let bevy_tokens = generate_bevy();

    let attr_implementations = generate_attr_implementations(&model);
    let element_tokens = generate_elements(&model);
    let mutation_tokens = generate_mutations(&model)?;

    Ok(quote! {
        #bevy_tokens

        #attr_implementations

        #element_tokens

        #mutation_tokens

        pub mod prelude {
            pub use super::*;
            pub use dioxus;
            pub use dioxus::prelude::{Event as DioxusEvent, *};
        }
    })
}
