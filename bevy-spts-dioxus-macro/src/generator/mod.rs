mod template_node;
mod dioxus_elements;
mod attribute_fns;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parser::Model;

pub fn generate(model: &Model) -> syn::Result<TokenStream> {
    let attribute_fns = attribute_fns::generate_attribute_fns(model);
    let template_node = template_node::generate_template_node(model);
    let template_node_impl = template_node::implement_template_node(model);
    let dioxus_elements = dioxus_elements::generate_dioxus_elements(model);

    let module_ident = model.module_ident.clone();
    let pass_through_items: TokenStream = model.pass_through_items.iter().map(|v| v.to_token_stream()).collect();

    Ok(quote! {
        pub mod #module_ident {
            #dioxus_elements
            #attribute_fns
            #template_node
            #template_node_impl

            #pass_through_items
        }
    })
}
