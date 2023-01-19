use crate::component::Component;
use std::any::TypeId;

mod detection;
mod iter;
mod record;

pub(crate) use detection::ChangeDetection;
pub use iter::*;
pub(crate) use record::*;

pub enum ComponentChange {
    Added(Box<dyn Component>),
    Removed(TypeId),
}
