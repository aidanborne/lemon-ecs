use std::{any::TypeId, cell::RefCell, collections::HashMap};

use crate::{
    component::Component,
    query::{Query, QueryComparison, result::QueryResult, queryable::Queryable},
    storage::{bundle::ComponentBundle, entities::EntityStorage},
};

/// Manages the entity storages. Each storage is an archetype.
/// 0 is the default archetype, which is always present.
pub struct World {
    archetype_storage: Vec<EntityStorage>,
    query_storage_cache: RefCell<HashMap<Query, Vec<usize>>>,
    query_comp_cache: RefCell<HashMap<(Query, Query), QueryComparison>>,
    available_ids: Vec<usize>,
    next_id: usize,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetype_storage: vec![EntityStorage::new()],
            query_storage_cache: RefCell::new(HashMap::new()),
            query_comp_cache: RefCell::new(HashMap::new()),
            available_ids: Vec::new(),
            next_id: 0,
        }
    }

    fn compare_query(&self, query: &Query, other: &Query) -> QueryComparison {
        let mut cache = self.query_comp_cache.borrow_mut();

        match cache.get(&(query.clone(), other.clone())) {
            Some(comparison) => comparison.clone(),
            None => {
                let comparison = query.compare_to(other);
                cache.insert((query.clone(), other.clone()), comparison.clone());
                comparison
            }
        }
    }

    fn get_archetype_idx_all(&self, query: &Query) -> Vec<usize> {
        let mut cache = self.query_storage_cache.borrow_mut();

        match cache.get(query) {
            Some(indices) => indices.clone(),
            None => {
                let indices: Vec<_> = self
                    .archetype_storage
                    .iter()
                    .enumerate()
                    .filter(|(_, storage)| {
                        self.compare_query(query, &storage.get_query()).is_some()
                    })
                    .map(|(idx, _)| idx)
                    .collect();

                cache.insert(query.clone(), indices.clone());
                indices
            }
        }
    }

    /// Returns the index of the storage that matches the query exactly.
    /// Used to avoid having two mutable references to the same storage.
    fn get_archetype_idx_exact(&self, query: &Query) -> Option<usize> {
        self.get_archetype_idx_all(query)
            .iter()
            .copied()
            .find(|idx| {
                self.compare_query(query, &self.archetype_storage[*idx].get_query())
                    .is_exact()
            })
    }

    /// Returns the index of the storage that contains the entity.
    /// Prefer using get_entity_storage instead.
    fn get_entity_storage_idx(&self, entity: usize) -> Option<usize> {
        self.archetype_storage
            .iter()
            .position(|storage| storage.contains(entity))
    }

    fn get_entity_storage(&self, entity: usize) -> Option<&EntityStorage> {
        self.get_entity_storage_idx(entity)
            .map(|idx| &self.archetype_storage[idx])
    }

    fn get_entity_storage_mut(&mut self, entity: usize) -> Option<&mut EntityStorage> {
        self.get_entity_storage_idx(entity)
            .map(move |idx| &mut self.archetype_storage[idx])
    }

    fn push_storage(&mut self, storage: EntityStorage) {
        let idx = self.archetype_storage.len();
        self.archetype_storage.push(storage);

        let storage_query = self.archetype_storage[idx].get_query();

        for (query, indices) in self.query_storage_cache.borrow_mut().iter_mut() {
            if self.compare_query(query, &storage_query).is_some() {
                indices.push(idx);
            }
        }
    }

    pub fn spawn(&mut self) -> usize {
        let id = match self.available_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 1;
                id
            }
        };

        self.archetype_storage[0]
            .insert(id, ComponentBundle::new())
            .unwrap();
        id
    }

    pub fn despawn(&mut self, id: usize) {
        if let Some(archetype) = self.get_entity_storage_mut(id) {
            archetype.remove(id).unwrap();
            self.available_ids.push(id);
        }
    }

    pub fn has_component<T: 'static + Component>(&self, id: usize) -> bool {
        self.get_entity_storage(id)
            .map(|archetype| archetype.contains(id) && archetype.has_component(TypeId::of::<T>()))
            .unwrap_or(false)
    }

    pub fn add_component<T: 'static + Component>(&mut self, id: usize, component: T) {
        if let Some(curr_idx) = self.get_entity_storage_idx(id) {
            if self.archetype_storage[curr_idx].has_component(TypeId::of::<T>()) {
                self.archetype_storage[curr_idx]
                    .replace(id, component)
                    .unwrap();
            } else {
                let mut bundle = self.archetype_storage[curr_idx].remove(id).unwrap();
                bundle.insert(Box::new(component));

                match self.get_archetype_idx_exact(&bundle.get_query()) {
                    Some(idx) => {
                        self.archetype_storage[idx].insert(id, bundle).unwrap();
                    }
                    None => {
                        let mut archetype = self.archetype_storage[curr_idx].as_empty_with::<T>();
                        archetype.insert(id, bundle).unwrap();
                        self.push_storage(archetype);
                    }
                }
            }
        }
    }

    pub fn remove_component<T: 'static + Component>(&mut self, id: usize) {
        if let Some(curr_idx) = self.get_entity_storage_idx(id) {
            if self.archetype_storage[curr_idx].has_component(TypeId::of::<T>()) {
                let mut bundle = self.archetype_storage[curr_idx].remove(id).unwrap();
                bundle.remove(&TypeId::of::<T>());

                match self.get_archetype_idx_exact(&bundle.get_query()) {
                    Some(idx) => {
                        self.archetype_storage[idx].insert(id, bundle).unwrap();
                    }
                    None => {
                        let mut archetype =
                            self.archetype_storage[curr_idx].as_empty_without::<T>();
                        archetype.insert(id, bundle).unwrap();
                        self.push_storage(archetype);
                    }
                }
            }
        }
    }

    pub fn get_component<T: 'static + Component>(&self, id: usize) -> Option<&T> {
        self.get_entity_storage(id)
            .and_then(|archetype| archetype.get_component::<T>(id).ok())
    }

    pub fn query<'a, T: 'static + Queryable<'a>>(&'a self) -> QueryResult<'a, T> {
        let query = T::get_query();
        let indices = self.get_archetype_idx_all(&query);
        QueryResult::new(&self.archetype_storage, indices)
    }
}
