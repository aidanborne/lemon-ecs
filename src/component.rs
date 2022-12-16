use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{query::Query, sparse_set::SparseSet};

pub trait Component {
    fn as_any(self) -> Box<dyn Any>;
    fn get_type_id(&self) -> TypeId;
}

pub trait ComponentStorage {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn type_id(&self) -> TypeId;

    fn insert(&mut self, id: usize, component: Box<dyn Component>);
    fn remove(&mut self, id: usize) -> Option<Box<dyn Component>>;

    fn as_empty_box(&self) -> Box<dyn ComponentStorage>;
}

impl<T: 'static + Component> ComponentStorage for SparseSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn insert(&mut self, id: usize, component: Box<dyn Component>) {
        if (*component).get_type_id() == TypeId::of::<T>() {
            let component = unsafe { Box::from_raw(Box::into_raw(component) as *mut T) };
            self.insert(id, *component);
        }
    }

    fn remove(&mut self, id: usize) -> Option<Box<dyn Component>> {
        let component = self.remove(id)?;
        Some(Box::new(component))
    }

    fn as_empty_box(&self) -> Box<dyn ComponentStorage> {
        Box::new(Self::new())
    }
}

pub struct ComponentBundle {
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl ComponentBundle {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, component: Box<dyn Component>) {
        self.components.insert(component.get_type_id(), component);
    }

    pub fn remove(&mut self, type_id: &TypeId) -> Option<Box<dyn Component>> {
        self.components.remove(type_id)
    }

    pub fn get_query(&self) -> Query {
        Query::new(self.components.keys().cloned().collect())
    }
}
