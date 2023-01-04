use crate::component::Component;

use super::downcast::{AsAny, Downcast};

/// A trait for component vectors that allows for replacing and swapping components.
/// Indices may not correspond to entity IDs.
pub trait ComponentVec: AsAny {
    /// Replaces the component at the given index with the given component.
    /// Returns the component that was previously at the given index.
    /// If the index is out of bounds, the component is appended to the vector.
    fn replace_index(
        &mut self,
        idx: usize,
        value: Box<dyn Component>,
    ) -> Option<Box<dyn Component>>;

    /// Swap-removes the component at the given index and returns it.
    fn swap_remove(&mut self, idx: usize) -> Box<dyn Component>;
}

impl<T: 'static + Component> AsAny for Vec<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<T: 'static + Component> ComponentVec for Vec<T> {
    fn replace_index(
        &mut self,
        idx: usize,
        component: Box<dyn Component>,
    ) -> Option<Box<dyn Component>> {
        if let Ok(component) = component.downcast::<T>() {
            if self.len() > idx {
                let removed = std::mem::replace::<T>(&mut self[idx], *component);
                return Some(Box::new(removed));
            } else {
                self.push(*component);
                return None;
            }
        }

        None
    }

    #[inline]
    fn swap_remove(&mut self, idx: usize) -> Box<dyn Component> {
        Box::new(self.swap_remove(idx))
    }
}
