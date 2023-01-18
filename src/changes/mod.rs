use crate::component::Component;
use std::any::TypeId;

mod detection;
mod record;

pub(crate) use detection::*;
pub(crate) use record::*;

pub enum ComponentChange {
    Added(Box<dyn Component>),
    Removed(TypeId),
}
