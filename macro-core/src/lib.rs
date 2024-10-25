extern crate proc_macro;

// Parses the input token stream into a model
mod parser;
// Generates the output token stream from the model
mod generator;

use syn::parse2;
use parser::Model;

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
    println!("model: {model:#?}");
    // proc_macro::TokenStream::from(input)
    todo!()
}

