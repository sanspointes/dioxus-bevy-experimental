extern crate proc_macro;

// Parses the input token stream into a model
pub(crate) mod parser;
pub(crate) mod generator;

use parser::Model;
use syn::parse2;

/// Defines a dioxus renderer for bevy.
///
/// * `input`:
#[proc_macro_attribute]
pub fn dioxus_bevy(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let model = match parse2::<Model>(input) {
        Ok(model) => model,
        Err(err) => return err.to_compile_error().into(),
    };

    let tokens = match generator::generate(&model) {
        Ok(tokens) => tokens,
        Err(err) => return err.to_compile_error().into(),
    };

    tokens.into()
}

#[proc_macro_attribute]
/// Defines a new element to be used in the dioxus bevy renderer.
///
/// ## Example
/// ```rust
/// #[define_attr]
/// fn position_x_attr(mut entity_mut: EntityWorldMut, value: AttributeValue) {
///     entity_mut.get::<Transform>().unwrap().translation.x = value.as_f32().unwrap_or(0.);
/// }
///
/// pub mod dioxus_elements {
///     #[define_element]
///     struct my_element {
///         #[attr]
///         position_x: position_x_attr,
///     }
/// }
/// ```
pub fn define_element(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut out = proc_macro::TokenStream::new();
    out.extend(attr);
    out.extend(input);
    out
}

#[proc_macro_attribute]
/// Defines an attribute that can later be used in an element definition.
///
/// ## Example
/// ```rust
/// #[define_attr]
/// fn position_x_attr(mut entity_mut: EntityWorldMut, value: AttributeValue) {
///     entity_mut.get::<Transform>().unwrap().translation.x = value.as_f32().unwrap_or(0.);
/// }
///
/// pub mod dioxus_elements {
///     #[define_element]
///     struct my_element {
///         #[attr]
///         position_x: position_x_attr,
///     }
/// }
/// ```
pub fn define_attr(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut out = proc_macro::TokenStream::new();
    out.extend(attr);
    out.extend(input);
    out
}
