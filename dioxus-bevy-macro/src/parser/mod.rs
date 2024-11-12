pub(crate) mod element_definition;

use element_definition::DioxusElementsModule;
use std::collections::HashMap;
use syn::{parse::Parse, spanned::Spanned, Ident, Item, ItemFn, ItemMod};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Model {
    pub module_ident: Ident,
    pub attribute_definitions: HashMap<String, ItemFn>,
    pub dioxus_elements_module: DioxusElementsModule,
    pub pass_through_items: Vec<Item>,
}

const MISSING_DIOXUS_ELEMENTS_MODULE_ERROR_MESSAGE: &str = r#"Missing dioxus_elements module.
#[define_attr]
fn position_x_attr(mut entity_mut: EntityWorldMut, value: AttributeValue) {
    entity_mut.get::<Transform>().unwrap().translation.x = value.as_f32().unwrap_or(0.);
}

pub mod dioxus_elements {
    #[define_element]
    struct my_element {
        #[attr]
        position_x: position_x_attr,
    }
}
"#;

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let module = input.parse::<ItemMod>().map_err(|err| {
            syn::Error::new(
                err.span(),
                "Expected a module `mod my_dioxus_bevy_adapter { ... }`.",
            )
        })?;

        let module_ident = module.ident.clone();
        let mut attribute_definitions = HashMap::new();
        let mut dioxus_elements_module: Option<DioxusElementsModule> = None;
        let mut pass_through_items = vec![];

        let Some((_, content_items)) = module.content else {
            return Err(syn::Error::new(module.span(), "Expected module content."));
        };

        for item in content_items {
            match &item {
                syn::Item::Fn(item_fn) => {
                    if item_fn
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("define_attr"))
                    {
                        attribute_definitions
                            .insert(item_fn.sig.ident.to_string(), item_fn.clone());
                        continue;
                    }
                }
                syn::Item::Mod(item_mod) => {
                    let is_dioxus_elements_module = item_mod.ident == "dioxus_elements";
                    let has_public_visibility = matches!(item_mod.vis, syn::Visibility::Public(_));
                    if is_dioxus_elements_module {
                        if !has_public_visibility {
                            return Err(syn::Error::new(
                                item_mod.vis.span(),
                                "The 'dioxus_elements' module must have public visibility.",
                            ));
                        }
                        dioxus_elements_module = Some(DioxusElementsModule::try_from(item_mod)?);
                        continue;
                    }
                }
                _ => {}
            }
            pass_through_items.push(item)
        }

        let dioxus_elements_module = dioxus_elements_module.ok_or_else(|| {
            syn::Error::new(input.span(), MISSING_DIOXUS_ELEMENTS_MODULE_ERROR_MESSAGE)
        })?;

        Ok(Model {
            module_ident,
            attribute_definitions,
            dioxus_elements_module,
            pass_through_items,
        })
    }
}
