use std::any::Any;

use crate::component::Component;

/// A trait for a vector of components.
/// The indices of the vector do not correspond to the entity id.
pub trait ComponentVec {
    fn as_any(&self) -> &dyn Any;

    fn insert(&mut self, id: usize, component: Box<dyn Component>);
    fn remove(&mut self, id: usize) -> Box<dyn Component>;

    fn as_empty_boxed(&self) -> Box<dyn ComponentVec>;
}

impl<T: 'static + Component> ComponentVec for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn insert(&mut self, idx: usize, component: Box<dyn Component>) {
        if let Ok(component) = component.downcast::<T>() {
            if idx >= self.capacity() {
                self.reserve(idx - self.capacity() + 1);
            }

            self.insert(idx, *component);
        }
    }

    fn remove(&mut self, idx: usize) -> Box<dyn Component> {
        Box::new(self.swap_remove(idx))
    }

    fn as_empty_boxed(&self) -> Box<dyn ComponentVec> {
        Box::new(Self::new())
    }
}
