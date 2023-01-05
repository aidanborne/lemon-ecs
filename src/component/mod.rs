use std::any::Any;

use crate::storage::{components::ComponentVec, downcast::AsAny};

pub mod bundle;
pub(crate) mod changes;

pub trait Component: AsAny {
    fn get_storage(&self) -> Box<dyn ComponentVec>;
}

impl<T: Component> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub mod prelude {
    pub use super::{bundle::*, Component};
}
