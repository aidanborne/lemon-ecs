use std::{any::TypeId, collections::HashMap};

use crate::{
    component::{Component, ComponentVec},
    sparse_set::SparseSet,
};

use super::EntityId;

pub struct Archetype {
    entities: SparseSet<()>,
    components: HashMap<TypeId, Box<dyn ComponentVec>>,
}

impl Archetype {
    pub fn from_components(components: &[Box<dyn Component>]) -> Self {
        let mut hash_map = HashMap::new();

        for component in components.iter() {
            hash_map.insert(component.as_any().type_id(), component.as_empty_vec());
        }

        Self {
            entities: SparseSet::new(),
            components: hash_map,
        }
    }

    /// Replaces the component of type `T` with the given component.
    pub fn replace_component(
        &mut self,
        id: EntityId,
        component: Box<dyn Component>,
    ) -> Box<dyn Component> {
        if let Some(idx) = self.entities.dense_idx(id) {
            self.components
                .get_mut(&component.as_any().type_id())
                .and_then(|components| components.replace(idx, component))
                .unwrap_or_else(|| panic!("Entity did not have the given component."))
        } else {
            panic!("Entity does not exist in this archetype.")
        }
    }

    pub fn insert(&mut self, id: EntityId, components: Vec<Box<dyn Component>>) {
        if !self.entities.contains(id) {
            self.entities.insert(id, ());
        }

        let dense_idx = self.entities.dense_idx(id).unwrap();

        for component in components {
            if let Some(storage) = self.components.get_mut(&component.as_any().type_id()) {
                storage.replace(dense_idx, component);
            }
        }
    }

    pub fn remove(&mut self, id: EntityId) -> Option<Vec<Box<dyn Component>>> {
        if let Some(idx) = self.entities.dense_idx(id) {
            let mut components = Vec::new();

            for (_type_id, storage) in self.components.iter_mut() {
                components.push(storage.swap_remove(idx));
            }

            self.entities.remove(id);
            Some(components)
        } else {
            None
        }
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains(id)
    }

    #[inline]
    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.components.contains_key(&type_id)
    }

    /// Returns a reference to the component of type `T` at the given dense index.
    pub fn get_component_dense<T: 'static + Component>(&self, idx: usize) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        if let Some(boxed_components) = self.components.get(&type_id) {
            if let Some(components) = boxed_components.downcast_ref::<Vec<T>>() {
                return components.as_slice().get(idx);
            }
        }

        None
    }

    pub fn get_component<T: 'static + Component>(&self, id: EntityId) -> Option<&T> {
        if let Some(idx) = self.entities.dense_idx(id) {
            return self.get_component_dense::<T>(idx);
        }

        None
    }

    pub(crate) fn entities(&self) -> &SparseSet<()> {
        &self.entities
    }
}
