use std::{any::TypeId, cell::RefCell, collections::HashMap, iter::Peekable};

use crate::{
    query::{QueryComparison, QueryIter}, storage::{entities::EntityStorage, bundle::ComponentBundle, sparse_set::Keys},
};

use super::{
    component::Component,
    query::{Query, Queryable},
};

pub struct World {
    next_id: usize,
    available_ids: Vec<usize>,
    archetypes: Vec<EntityStorage>,
    query_cache: RefCell<HashMap<(Query, Query), QueryComparison>>,
    archetype_cache: RefCell<HashMap<Query, Vec<usize>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            available_ids: Vec::new(),
            archetypes: vec![EntityStorage::new()],
            query_cache: RefCell::new(HashMap::new()),
            archetype_cache: RefCell::new(HashMap::new()),
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

        self.archetypes[0]
            .insert(id, ComponentBundle::new())
            .unwrap();
        id
    }

    pub fn despawn(&mut self, id: usize) {
        if let Some(archetype_idx) = self.archetype_of_idx(id) {
            self.archetypes[archetype_idx].remove(id).unwrap();
            self.available_ids.push(id);
        }
    }

    fn archetype_of_idx(&self, id: usize) -> Option<usize> {
        self.archetypes
            .iter()
            .enumerate()
            .find(|archetype| archetype.1.contains(id))
            .map(|archetype| archetype.0)
    }

    fn archetype_of(&self, id: usize) -> Option<&EntityStorage> {
        self.archetype_of_idx(id).map(|idx| &self.archetypes[idx])
    }

    /// Caches the results of queries to avoid unnecessary allocations
    pub(crate) fn compare_query(&self, query_a: &Query, query_b: &Query) -> QueryComparison {
        let mut query_cache = self.query_cache.borrow_mut();
        let key = (query_a.clone(), query_b.clone());

        if let Some(result) = query_cache.get(&key) {
            *result
        } else {
            let result = query_a.compare_to(query_b);
            query_cache.insert(key, result);
            result
        }
    }

    fn find_archetype(&self, bundle: &ComponentBundle) -> Option<usize> {
        let bundle_query = bundle.get_query();

        self.archetypes
            .iter()
            .enumerate()
            .find(|archetype| {
                self.compare_query(&archetype.1.get_query(), &bundle_query)
                    .is_exact()
            })
            .map(|archetype| archetype.0)
    }

    pub fn has_component<T: 'static + Component>(&self, id: usize) -> bool {
        self.archetype_of(id)
            .map(|archetype| archetype.has_component(TypeId::of::<T>()) && archetype.contains(id))
            .unwrap_or(false)
    }

    fn add_archetype(&mut self, archetype: EntityStorage) -> usize {
        let idx = self.archetypes.len();
        self.archetypes.push(archetype);

        for (query, archetypes) in self.archetype_cache.borrow_mut().iter_mut() {
            if self
                .compare_query(query, &self.archetypes[idx].get_query())
                .is_some()
            {
                archetypes.push(idx);
            }
        }

        idx
    }

    pub fn add_component<T: 'static + Component>(&mut self, id: usize, component: T) {
        if let Some(archetype_idx) = self.archetype_of_idx(id) {
            let archetype = &mut self.archetypes[archetype_idx];

            if archetype.has_component(TypeId::of::<T>()) {
                archetype.replace(id, component).unwrap();
            } else {
                let mut bundle = archetype.remove(id).unwrap();
                bundle.insert(Box::new(component));

                let idx = self.find_archetype(&bundle).unwrap_or_else(|| {
                    self.add_archetype(self.archetypes[archetype_idx].as_empty_with::<T>())
                });

                self.archetypes[idx].insert(id, bundle).unwrap();
            }
        }
    }

    pub fn remove_component<T: 'static + Component>(&mut self, id: usize) {
        if let Some(archetype_idx) = self.archetype_of_idx(id) {
            let archetype = &mut self.archetypes[archetype_idx];

            if archetype.has_component(TypeId::of::<T>()) {
                let mut bundle = archetype.remove(id).unwrap();
                bundle.remove(&TypeId::of::<T>());

                let idx = self.find_archetype(&bundle).unwrap_or_else(|| {
                    self.add_archetype(self.archetypes[archetype_idx].as_empty_without::<T>())
                });

                self.archetypes[idx].insert(id, bundle).unwrap();
            }
        }
    }

    pub fn get_component<T: 'static + Component>(&self, id: usize) -> Option<&T> {
        let archetype = self.archetype_of(id)?;
        archetype.get_component(id).ok()
    }

    pub fn iter(&self, query: Query) -> EntityIter {
        let mut cache = self.archetype_cache.borrow_mut();

        if cache.get(&query).is_none() {
            let archetypes: Vec<_> = self
                .archetypes
                .iter()
                .enumerate()
                .filter(|archetype| {
                    self.compare_query(&query, &archetype.1.get_query())
                        .is_some()
                })
                .map(|archetype| archetype.0)
                .collect();

                cache.insert(query.clone(), archetypes);
            }
        
            EntityIter::new(&self.archetypes, cache.get(&query).unwrap().clone())
    }

    pub fn query<'a, T: 'static + Queryable<'a>>(&'a self) -> QueryIter<'a, T> {
        QueryIter::new(self)
    }
}

struct EntityStorageIter<'a> {
    archetypes: &'a [EntityStorage],
    indices: std::vec::IntoIter<usize>,
}

impl<'a> Iterator for EntityStorageIter<'a> {
    type Item = &'a EntityStorage;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|idx| &self.archetypes[idx])
    }
}

pub struct EntityIter<'a> {
    archetypes: Peekable<EntityStorageIter<'a>>,
    entities: Option<Keys<'a, usize>>,
}

impl<'a> EntityIter<'a> {
    fn new(archetypes: &'a Vec<EntityStorage>, indices: Vec<usize>) -> Self{
        Self {
            archetypes: EntityStorageIter {
                archetypes: archetypes.as_slice(),
                indices: indices.into_iter()
            }
            .peekable(),
            entities: None,
        }
    }

    pub(crate) fn get_archetype(&mut self) -> Option<&'a EntityStorage> {
        self.archetypes.peek().copied()
    }
}

impl<'a> Iterator for EntityIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entities) = &mut self.entities {
                if let Some(id) = entities.next() {
                    return Some(id);
                }

                self.archetypes.next();
            }

            if let Some(archetype) = self.archetypes.peek() {
                self.entities = Some(archetype.iter());
            } else {
                return None;
            }
        }
    }
}
