use std::any::{Any, TypeId};

use crate::component::Component;

use super::sparse_set::SparseSet;

pub trait ComponentStorage {
  fn as_any(&self) -> &dyn Any;

  fn insert(&mut self, id: usize, component: Box<dyn Any>);
  fn remove(&mut self, id: usize) -> Option<Box<dyn Any>>;

  fn as_empty_boxed(&self) -> Box<dyn ComponentStorage>;
}

impl<T: 'static + Component> ComponentStorage for SparseSet<T> {
  fn as_any(&self) -> &dyn Any {
      self
  }

  fn insert(&mut self, id: usize, component: Box<dyn Any>) {
      if (*component).type_id() == TypeId::of::<T>() {
         self.insert(id, *component.downcast::<T>().unwrap());
      }
  }

  fn remove(&mut self, id: usize) -> Option<Box<dyn Any>> {
      self.remove(id).map(|component| Box::new(component) as Box<dyn Any>)
  }

  fn as_empty_boxed(&self) -> Box<dyn ComponentStorage> {
      Box::new(Self::new())
  }
}
