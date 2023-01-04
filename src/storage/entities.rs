use std::{any::TypeId, collections::HashMap};

use crate::component::{bundle::ComponentBundle, Component};

use super::{components::ComponentVec, sparse_set::SparseSet};

pub struct EntitySparseSet {
    entities: SparseSet<usize>,
    components: HashMap<TypeId, Box<dyn ComponentVec>>,
}

impl EntitySparseSet {
    pub fn new() -> Self {
        Self {
            entities: SparseSet::new(),
            components: HashMap::new(),
        }
    }

    pub fn from_bundle(bundle: &ComponentBundle) -> Self {
        let mut components = HashMap::new();

        for component in bundle.iter() {
            components.insert(component.as_any().type_id(), component.get_storage());
        }

        Self {
            entities: SparseSet::new(),
            components,
        }
    }

    pub fn replace_component(
        &mut self,
        id: usize,
        component: Box<dyn Component>,
    ) -> Option<Box<dyn Component>> {
        if let Some(idx) = self.entities.dense_idx(id) {
            if let Some(storage) = self.components.get_mut(&component.as_any().type_id()) {
                return storage.replace_index(idx, component);
            }
        }

        None
    }

    pub fn insert(&mut self, id: usize, bundle: ComponentBundle) {
        if !self.entities.contains(id) {
            self.entities.insert(id, self.entities.len());
        }

        for component in bundle {
            if let Some(storage) = self.components.get_mut(&component.as_any().type_id()) {
                storage.replace_index(self.entities.dense_idx(id).unwrap(), component);
            }
        }
    }

    pub fn remove(&mut self, id: usize) -> Option<ComponentBundle> {
        if let Some(idx) = self.entities.dense_idx(id) {
            let mut bundle = ComponentBundle::new();

            for (_type_id, storage) in self.components.iter_mut() {
                bundle.push(storage.swap_remove(idx));
            }

            self.entities.remove(id);
            Some(bundle)
        } else {
            None
        }
    }

    pub fn contains(&self, id: usize) -> bool {
        self.entities.contains(id)
    }

    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.components.contains_key(&type_id)
    }

    /// Returns a reference to the component of type `T` at the given dense index.
    pub fn get_component<T: 'static + Component>(&self, idx: usize) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        if let Some(component_storage) = self.components.get(&type_id) {
            if let Some(components) = component_storage.as_any().downcast_ref::<Vec<T>>() {
                return components.get(idx);
            }
        }

        None
    }

    pub fn type_ids<T: FromIterator<TypeId>>(&self) -> T {
        self.components.keys().cloned().collect()
    }

    pub fn entities(&self) -> super::sparse_set::Iter<usize> {
        self.entities.iter()
    }
}
