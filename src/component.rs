use std::any::TypeId;

use crate::storage::components::ComponentVec;

use lemon_ecs_macros::impl_tuple_bundle;

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

pub type ComponentBundle = Vec<Box<dyn Component>>;

pub trait Bundleable {
    fn bundle(self) -> Vec<Box<dyn Component>>;
}

impl<T> Bundleable for T
where
    T: 'static + Component,
{
    fn bundle(self) -> Vec<Box<dyn Component>> {
        vec![Box::new(self)]
    }
}

impl Bundleable for ComponentBundle {
    fn bundle(self) -> Vec<Box<dyn Component>> {
        self
    }
}

impl_tuple_bundle!(0..16);
