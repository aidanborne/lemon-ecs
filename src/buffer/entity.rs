use std::{any::TypeId, collections::HashMap};

use crate::{
    component::{Bundle, Component, TypeBundle},
    entities::EntityId,
    world::ComponentChange,
};

pub struct ExistingEntityBuffer(EntityId, HashMap<TypeId, ComponentChange>);

impl ExistingEntityBuffer {
    pub fn new(id: EntityId) -> Self {
        Self(id, HashMap::new())
    }

    pub fn id(&self) -> EntityId {
        self.0
    }

    pub fn insert(&mut self, components: impl Bundle) -> &mut Self {
        for component in components.components() {
            self.1.insert(
                (*component).as_any().type_id(),
                ComponentChange::Insert(component),
            );
        }

        self
    }

    pub fn remove<T: TypeBundle>(&mut self) -> &mut Self {
        for type_id in T::type_ids() {
            self.1.insert(type_id, ComponentChange::Remove(type_id));
        }

        self
    }

    pub fn into_changes(self) -> Vec<ComponentChange> {
        self.1.into_values().collect()
    }
}

#[repr(transparent)]
pub struct SpawnedEntityBuffer(HashMap<TypeId, Box<dyn Component>>);

impl SpawnedEntityBuffer {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, components: impl Bundle) -> &mut Self {
        for component in components.components() {
            self.0.insert((*component).as_any().type_id(), component);
        }

        self
    }

    pub fn remove<T: TypeBundle>(&mut self) -> &mut Self {
        for type_id in T::type_ids() {
            self.0.remove(&type_id);
        }

        self
    }

    pub fn into_components(self) -> Vec<Box<dyn Component>> {
        self.0.into_values().collect()
    }
}

impl Default for SpawnedEntityBuffer {
    fn default() -> Self {
        Self::new()
    }
}
