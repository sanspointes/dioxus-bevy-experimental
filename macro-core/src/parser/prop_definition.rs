use syn::{parenthesized, parse::Parse, ExprClosure, Ident, Token};

#[allow(dead_code)]
#[derive(Debug)]
pub struct PropDefinition {
    pub ident: Ident,
    pub ty: Ident,
    pub applier_fn: ExprClosure,
}

impl Parse for PropDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let ident = content.parse::<Ident>()?;
        content.parse::<Token![,]>()?;
        let ty = content.parse::<Ident>()?;
        content.parse::<Token![,]>()?;

        let applier_fn = content.parse::<ExprClosure>()?;

        Ok(Self {
            ident,
            ty,
            applier_fn,
        })
    }
}

#[allow(dead_code)]
pub enum RendererAttribute {
    PropDefinition(PropDefinition),
}

impl Parse for RendererAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![@]>()?;
        let attr: Ident = input.parse()?;
        let mut result: Option<RendererAttribute> = None;
        if attr == "define_prop" {
            let contents = input.parse::<PropDefinition>()?;
            result = Some(RendererAttribute::PropDefinition(contents));
            input.parse::<Token![;]>()?;
        }

        if let Some(result) = result {
            Ok(result)
        } else {
            Err(syn::Error::new(attr.span(), "Unknown definition.  Expected 'prop'."))
        }
    }
}
