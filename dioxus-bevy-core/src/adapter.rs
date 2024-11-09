use std::any::Any;

use bevy_ecs::{
    entity::Entity,
    world::{EntityWorldMut, World},
};
use dioxus::{dioxus_core::AttributeValue, prelude::{IntoAttributeValue, TemplateNode}};

/// Intermediary format from Dioxus's [Template] that can be spawned into the world.
pub trait DioxusBevyTemplateNode: Send + Sync + Clone + 'static {
    fn from_dioxus(node: &TemplateNode) -> Self;
    fn spawn(&self, world: &mut World) -> Entity;
    fn apply_attribute(entity_mut: EntityWorldMut, name: &'static str, value: &AttributeValue);
}

#[allow(dead_code)]
pub trait AttributeValueHelpers {
    fn with_concrete<T: 'static, U>(&self, scope: impl FnOnce(&T)-> U) -> Option<U>;
    fn as_concrete<T: 'static>(&self) -> Option<&T>;
    fn as_f32(&self) -> Option<f32>;
    fn as_f64(&self) -> Option<f64>;
    fn as_string(&self) -> Option<&String>;
    fn as_i64(&self) -> Option<i64>;
    fn as_i32(&self) -> Option<i32>;
    fn as_bool(&self) -> Option<bool>;
}

impl AttributeValueHelpers for AttributeValue {
    fn with_concrete<T: 'static, U>(&self, scope: impl FnOnce(&T)-> U) -> Option<U> {
        match self {
            Self::Any(boxed_any) => {
                let value = boxed_any.as_any().downcast_ref::<T>()?;
                Some(scope(value))
            },
            _ => None
        }
    }

    fn as_concrete<T: 'static>(&self) -> Option<&T> {
        match self {
            Self::Any(boxed_any) => boxed_any.as_any().downcast_ref::<T>(),
            _ => None
        }
        
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Text(value) => value.parse::<f64>().ok(),
            Self::Int(value) => Some(*value as f64),
            _ => None
        }
    }

    fn as_f32(&self) -> Option<f32> {
        match self {
            Self::Float(value) => Some(*value as f32),
            Self::Text(value) => value.parse::<f32>().ok(),
            Self::Int(value) => Some(*value as f32),
            _ => None
        }
    }

    fn as_string(&self) -> Option<&String> {
        match self {
            Self::Text(value) => Some(value),
            _ => None
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            Self::Text(value) => value.parse::<i64>().ok(),
            _ => None
        }
    }

    fn as_i32(&self) -> Option<i32> {
        match self {
            Self::Int(value) => Some(*value as i32),
            Self::Text(value) => value.parse::<i32>().ok(),
            _ => None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            Self::Text(value) => value.parse::<bool>().ok(),
            _ => None,
        }
    }
}

/// Wrapped Attribute, required to implement IntoAttributeValue for external types.
pub struct WA<T: Any + PartialEq>(pub T);

impl<T: Any + PartialEq> IntoAttributeValue for WA<T> {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Any(Box::new(self.0))
    }
}

/// Implement this trait on a #\[define_element\] struct to spawn it.
pub trait DioxusBevyElement {
    fn spawn(world: &mut World) -> EntityWorldMut;
}
