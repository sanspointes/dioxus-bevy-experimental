use syn::{parse::Parse, ItemFn};

#[derive(Debug)]
pub struct AttrDefinition {
    item_fn: ItemFn,
}

impl Parse for AttrDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_fn = input.parse::<ItemFn>()?;

        Ok(Self { item_fn })
    }
}
