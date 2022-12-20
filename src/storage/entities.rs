use std::{any::TypeId, collections::HashMap};

use crate::{component::Component, query::archetype::Archetype};

use super::{bundle::ComponentBundle, components::ComponentVec, sparse_set::SparseSet};

pub struct EntityStorage {
    entities: SparseSet<usize>,
    components: HashMap<TypeId, Box<dyn ComponentVec>>,
}

impl EntityStorage {
    pub fn new() -> Self {
        Self {
            entities: SparseSet::new(),
            components: HashMap::new(),
        }
    }

    pub fn from_bundle(bundle: &ComponentBundle) -> Self {
        let mut components = HashMap::new();

        for component in bundle.iter() {
            components.insert(component.component_id(), component.create_storage());
        }

        Self {
            entities: SparseSet::new(),
            components,
        }
    }

    pub fn replace<T: 'static + Component>(&mut self, id: usize, component: T) {
        if let Some(idx) = self.entities.dense_idx(id) {
            if let Some(storage) = self.components.get_mut(&TypeId::of::<T>()) {
                storage.insert(idx, Box::new(component));
            }
        }
    }

    pub fn insert(&mut self, id: usize, bundle: ComponentBundle) {
        if !self.entities.contains(id) {
            self.entities.insert(id, id);
        }

        for component in bundle {
            if let Some(storage) = self.components.get_mut(&component.component_id()) {
                storage.insert(self.entities.dense_idx(id).unwrap(), component);
            }
        }
    }

    pub fn remove(&mut self, id: usize) -> Option<ComponentBundle> {
        if let Some(idx) = self.entities.dense_idx(id) {
            let mut bundle = ComponentBundle::new();

            for (_type_id, storage) in self.components.iter_mut() {
                bundle.push(storage.remove(idx));
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

    pub fn get_component<T: 'static + Component>(&self, idx: usize) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        if let Some(component_storage) = self.components.get(&type_id) {
            if let Some(component) = component_storage.as_any().downcast_ref::<Vec<T>>() {
                return component.get(idx);
            }
        }

        None
    }

    pub fn get_archetype(&self) -> Archetype {
        self.components.keys().cloned().collect()
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self.entities.iter())
    }
}

#[derive(Clone, Copy)]
pub struct Entity {
    // The id of the entity
    id: usize,
    // The index of the entity in the storage
    idx: usize,
}

impl Entity {
    pub fn new(id: usize, idx: usize) -> Self {
        Self { id, idx }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

impl std::ops::Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

pub struct Iter<'a> {
    entities: super::sparse_set::Iter<'a, usize>,
    idx: usize,
}

impl<'a> Iter<'a> {
    pub fn new(entities: super::sparse_set::Iter<'a, usize>) -> Self {
        Self { entities, idx: 0 }
    }
}

impl Iterator for Iter<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.entities.next() {
            let idx = self.idx;
            self.idx += 1;
            Some(Entity::new(id.key, idx))
        } else {
            None
        }
    }
}
