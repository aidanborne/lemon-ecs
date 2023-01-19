use crate::{
    component::Component,
    downcast::{impl_downcast, AsAny},
};

/// A trait for component vectors that allows for replacing and swapping components.
/// Indices may not correspond to entity IDs.
pub trait ComponentVec: AsAny {
    /// Replaces the component at the given index with the given value.
    fn replace(&mut self, idx: usize, value: Box<dyn Component>) -> Option<Box<dyn Component>>;

    /// Swap-removes the component at the given index and returns it.
    fn swap_remove(&mut self, idx: usize) -> Box<dyn Component>;

    /// Pushes the given component to the end of the vector.
    fn push(&mut self, value: Box<dyn Component>) -> usize;

    /// Returns a reference to the component at the given index.
    fn get(&self, idx: usize) -> Option<&dyn Component>;

    /// Creates a new empty component vector.
    fn clone_empty(&self) -> Box<dyn ComponentVec>;
}

impl_downcast!(dyn ComponentVec);

macro_rules! panic_incorrect_type {
    ($ty:ident) => {
        panic!("Incorrect type, expected {}", std::any::type_name::<$ty>())
    };
}

impl<T: 'static + Component> ComponentVec for Vec<T> {
    #[inline]
    fn replace(&mut self, idx: usize, component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        if let Ok(component) = component.downcast::<T>() {
            if self.len() > idx {
                let removed = std::mem::replace::<T>(&mut self[idx], *component);
                return Some(Box::new(removed));
            } else {
                self.push(*component);
                return None;
            }
        }

        panic_incorrect_type!(T);
    }

    #[inline]
    fn swap_remove(&mut self, idx: usize) -> Box<dyn Component> {
        Box::new(self.swap_remove(idx))
    }

    #[inline]
    fn push(&mut self, component: Box<dyn Component>) -> usize {
        if let Ok(component) = component.downcast::<T>() {
            self.push(*component);
            return self.len() - 1;
        }

        panic_incorrect_type!(T);
    }

    #[inline]
    fn get(&self, idx: usize) -> Option<&dyn Component> {
        self.as_slice().get(idx).map(|c| c as &dyn Component)
    }

    #[inline]
    fn clone_empty(&self) -> Box<dyn ComponentVec> {
        Box::<Vec<T>>::default()
    }
}
