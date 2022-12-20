use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    ops::{Index, IndexMut},
};

use crate::query::{archetype::Archetype, pattern::QueryPattern, Queryable};

use super::{bundle::ComponentBundle, entities::EntityStorage};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeId(usize);

pub struct ArchetypeArena {
    archetypes: Vec<EntityStorage>,
    bundle_cache: RefCell<HashMap<Archetype, usize>>,
    pattern_cache: RefCell<HashMap<QueryPattern, Vec<usize>>>,
    query_cache: RefCell<HashMap<TypeId, QueryPattern>>,
}

impl ArchetypeArena {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            bundle_cache: RefCell::default(),
            pattern_cache: RefCell::default(),
            query_cache: RefCell::default(),
        }
    }

    pub fn get_entity_archetype(&self, id: usize) -> Option<ArchetypeId> {
        self.archetypes
            .iter()
            .position(move |archetype| archetype.contains(id))
            .map(|idx| ArchetypeId(idx))
    }

    pub fn get_existing_archetype(&self, archetype: Archetype) -> Option<ArchetypeId> {
        let mut cache = self.bundle_cache.borrow_mut();

        match cache.get(&archetype) {
            Some(idx) => Some(ArchetypeId(*idx)),
            None => {
                let idx = self
                    .archetypes
                    .iter()
                    .position(|storage| storage.get_archetype() == archetype);

                if let Some(idx) = idx {
                    cache.insert(archetype, idx);
                    Some(ArchetypeId(idx))
                } else {
                    None
                }
            }
        }
    }

    pub fn get_bundle_archetype(&mut self, bundle: &ComponentBundle) -> ArchetypeId {
        let mut cache = self.bundle_cache.borrow_mut();

        let archetype = bundle
            .iter()
            .map(|component| component.component_id())
            .collect();

        match cache.get(&archetype) {
            Some(idx) => ArchetypeId(*idx),
            None => {
                let idx = self.archetypes.len();
                let storage = EntityStorage::from_bundle(bundle);

                self.archetypes.push(storage);

                for (pattern, indices) in self.pattern_cache.borrow_mut().iter_mut() {
                    if pattern.filter(&archetype) {
                        indices.push(idx);
                    }
                }

                cache.insert(archetype, idx);
                ArchetypeId(idx)
            }
        }
    }

    fn query_pattern(&self, pattern: &QueryPattern) -> Iter<'_> {
        let mut cache = self.pattern_cache.borrow_mut();

        let indices = match cache.get(pattern) {
            Some(indices) => indices.clone(),
            None => {
                let indices: Vec<_> = self
                    .archetypes
                    .iter()
                    .enumerate()
                    .filter(|(_, storage)| pattern.filter(&storage.get_archetype()))
                    .map(|(idx, _)| idx)
                    .collect();

                cache.insert(pattern.clone(), indices.clone());
                indices
            }
        };

        Iter::new(&self.archetypes, indices)
    }

    pub fn query_archetypes<'a, T: Queryable<'a>>(&'a self) -> Iter<'a> {
        let mut cache = self.query_cache.borrow_mut();

        let type_id = TypeId::of::<(T::Fetch, T::Filter)>();

        let pattern = match cache.get(&type_id) {
            Some(pattern) => pattern.clone(),
            None => {
                let pattern = T::get_pattern();
                cache.insert(type_id, pattern.clone());
                pattern
            }
        };

        self.query_pattern(&pattern)
    }
}

impl Index<ArchetypeId> for ArchetypeArena {
    type Output = EntityStorage;

    fn index(&self, index: ArchetypeId) -> &Self::Output {
        &self.archetypes[index.0]
    }
}

impl IndexMut<ArchetypeId> for ArchetypeArena {
    fn index_mut(&mut self, index: ArchetypeId) -> &mut Self::Output {
        &mut self.archetypes[index.0]
    }
}

pub struct Iter<'a> {
    archetypes: &'a [EntityStorage],
    indices: std::vec::IntoIter<usize>,
}

impl<'a> Iter<'a> {
    pub fn new(archetypes: &'a [EntityStorage], indices: Vec<usize>) -> Self {
        Self {
            archetypes,
            indices: indices.into_iter(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a EntityStorage;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|idx| &self.archetypes[idx])
    }
}
