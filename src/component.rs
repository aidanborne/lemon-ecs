use std::any::TypeId;

use crate::storage::components::ComponentVec;

pub trait Component {
    fn create_storage(&self) -> Box<dyn ComponentVec>;

    fn component_id(&self) -> TypeId;
}

impl dyn Component {
    pub fn downcast<T>(self: Box<Self>) -> Result<Box<T>, Box<Self>>
    where
        T: 'static + Component,
    {
        if TypeId::of::<T>() == self.component_id() {
            unsafe {
                let raw = Box::into_raw(self);
                Ok(Box::from_raw(raw as *mut T))
            }
        } else {
            Err(self)
        }
    }
}
