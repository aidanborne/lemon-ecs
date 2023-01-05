use std::any::Any;

use crate::storage::{downcast::AsAny, ComponentVec};

mod bundle;
mod changes;

pub use bundle::*;
pub(crate) use changes::*;

pub trait Component: AsAny {
    fn get_storage(&self) -> Box<dyn ComponentVec>;
}

impl<T: Component> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
