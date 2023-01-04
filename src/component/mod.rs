use crate::storage::{components::ComponentVec, downcast::AsAny};

pub mod bundle;
pub mod changes;

pub trait Component: AsAny {
    fn create_storage(&self) -> Box<dyn ComponentVec>;
}
