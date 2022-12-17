use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::query::Archetype;

pub struct ComponentBundle {
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentBundle {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, component: Box<dyn Any>) {
        self.components.insert((*component).type_id(), component);
    }

    pub fn remove(&mut self, type_id: &TypeId) -> Option<Box<dyn Any>> {
        self.components.remove(type_id)
    }

    pub fn get(&self, type_id: &TypeId) -> Option<&Box<dyn Any>> {
        self.components.get(type_id)
    }

    pub fn get_archetype(&self) -> Archetype {
        Archetype::new(self.components.keys().cloned().collect())
    }
}
