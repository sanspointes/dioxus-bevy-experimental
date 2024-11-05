extern crate proc_macro;

// Parses the input token stream into a model
pub(crate) mod parser;

use parser::Model;
use syn::parse2;

/// Defines a dioxus renderer for bevy.
///
/// * `input`:
#[proc_macro]
pub fn define_bevy_dioxus(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let model = match parse2::<Model>(input) {
        Ok(model) => model,
        Err(err) => return err.to_compile_error().into(),
    };

    dbg!(model);
    // let tokens = match generator::generate(model) {
    //     Ok(tokens) => tokens,
    //     Err(err) => return err.to_compile_error().into(),
    // };
    todo!()

    // tokens.into()
}
