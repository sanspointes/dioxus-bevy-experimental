use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parser::Model;

pub fn generate_attribute_fns(model: &Model) -> TokenStream {
    let attr_implementations: TokenStream = model
        .attribute_definitions
        .values()
        .map(|attr_def| attr_def.to_token_stream())
        .collect();

    quote! {
        #attr_implementations
    }
}
