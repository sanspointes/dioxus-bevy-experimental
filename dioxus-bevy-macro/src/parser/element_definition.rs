use syn::{spanned::Spanned, Field, Ident, Item, ItemMod, ItemStruct, Type};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ElementAttribute {
    pub field_ident: Ident,
    pub handler_ident: Ident,
}

impl TryFrom<&Field> for ElementAttribute {
    type Error = syn::Error;
    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        println!("ElementAttribute::try_from::<Field> {value:#?}");

        let field_ident = value.ident.clone().ok_or_else(|| syn::Error::new(value.span(), "Found field without an identifier.  This is usually caused by tuple structs, please convert to a normal struct."))?;

        match &value.ty {
            Type::Path(type_path) => {
                let handler_ident = type_path.path.get_ident().ok_or_else(||
                    syn::Error::new(type_path.span(), "Type path has multiple segments.  Keep the attributes defined in the same module so you can use a simple type path like `position`."))?;
                Ok(ElementAttribute {
                    field_ident,
                    handler_ident: handler_ident.clone(),
                })
            }
            other => Err(syn::Error::new(
                other.span(),
                "Expected a simple type path like `position`, found '{other}'.",
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ElementComponent {
    pub field_ident: Ident,
    pub component_type: Type,
}

impl TryFrom<&Field> for ElementComponent {
    type Error = syn::Error;
    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let field_ident = value.ident.clone().ok_or_else(|| syn::Error::new(value.span(), "Found field without an identifier.  This is usually caused by tuple structs, please convert to a normal struct."))?;
        Ok(Self {
            field_ident,
            component_type: value.ty.clone(),
        })
    }
}

#[derive(Debug)]
pub struct ElementDefinition {
    pub ident: Ident,
    pub attributes: Vec<ElementAttribute>,
    pub components: Vec<ElementComponent>,
}

const UNEXPECTED_FIELD_ERROR_MESSAGE: &str = r#"Unexpected field.  Currently only #[attr] and #[component] fields are supported, i.e.:
#[define_attr]
fn position_x_attr(mut entity_mut: EntityWorldMut, value: AttributeValue) {
    entity_mut.get::<Transform>().unwrap().translation.x = value.as_f32().unwrap_or(0.);
}

#[derive(Component, PartialEq)]
pub struct MyComp(1)

pub mod dioxus_elements {
    #[define_element]
    struct my_element {
        #[component]
        my_comp: MyComp,

        #[attr]
        position_x: position_x_attr,
    }
}
"#;

impl TryFrom<&ItemStruct> for ElementDefinition {
    type Error = syn::Error;

    fn try_from(value: &ItemStruct) -> syn::Result<Self> {
        println!("ElementDefinition::new() -> value: {value:#?}");

        let mut element_definition = ElementDefinition {
            ident: value.ident.clone(),
            attributes: vec![],
            components: vec![],
        };

        for field in value.fields.iter() {
            if field.attrs.iter().any(|attr| attr.path().is_ident("attr")) {
                element_definition
                    .attributes
                    .push(ElementAttribute::try_from(field)?);
            } else if field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("component"))
            {
                element_definition
                    .components
                    .push(ElementComponent::try_from(field)?);
            } else {
                return Err(syn::Error::new(
                    field.span(),
                    UNEXPECTED_FIELD_ERROR_MESSAGE,
                ));
            }
        }

        Ok(element_definition)
    }
}

#[derive(Debug)]
pub struct DioxusElementsModule {
    pub element_definitions: Vec<ElementDefinition>,
    pub pass_through_items: Vec<Item>,
}

impl TryFrom<&ItemMod> for DioxusElementsModule {
    type Error = syn::Error;
    fn try_from(value: &ItemMod) -> Result<Self, Self::Error> {
        let Some((_, content_items)) = &value.content else {
            return Err(syn::Error::new(value.span(), "Expected module content."));
        };

        let mut element_definitons = vec![];
        let mut pass_through_items = vec![];

        for item in content_items {
            #[allow(clippy::single_match)]
            match &item {
                syn::Item::Struct(item_struct) => {
                    if item_struct
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("define_element"))
                    {
                        let element_definition = ElementDefinition::try_from(item_struct)?;
                        element_definitons.push(element_definition);
                        continue;
                    }
                }
                _ => {}
            }
            pass_through_items.push(item.clone())
        }

        Ok(DioxusElementsModule {
            element_definitions: element_definitons,
            pass_through_items,
        })
    }
}
