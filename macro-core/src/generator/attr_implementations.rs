use quote::quote;
use syn::Ident;

use crate::parser::{prop_definition::PropDefinition, Model};

pub fn generate_attr_implementations(model: &Model) -> proc_macro2::TokenStream {
    model.props.values().map(|attr_definition| {
        let internal_ident = attr_definition.internal_ident();
        let PropDefinition { ident, ty, applier_fn } = attr_definition;
        println!("{internal_ident} applier_fn: {applier_fn:#?}");
        quote! {
            pub fn #internal_ident(entity_mut: bevy::ecs::world::EntityMut, value: #ty) {

            }
        }
    }).collect()
}
