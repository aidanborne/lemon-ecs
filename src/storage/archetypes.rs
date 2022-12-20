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

struct QueryCache {
    pattern: QueryPattern,
    archetypes: Vec<usize>,
}

#[derive(Default)]
pub struct ArchetypeArena {
    archetypes: Vec<EntityStorage>,
    bundle_cache: HashMap<Archetype, usize>,
    query_cache: RefCell<HashMap<TypeId, QueryCache>>,
}

impl ArchetypeArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_entity_archetype(&self, id: usize) -> Option<ArchetypeId> {
        self.archetypes
            .iter()
            .position(move |archetype| archetype.contains(id))
            .map(|idx| ArchetypeId(idx))
    }

    pub fn get_existing_archetype(&mut self, archetype: Archetype) -> Option<ArchetypeId> {
        match self.bundle_cache.get(&archetype) {
            Some(idx) => Some(ArchetypeId(*idx)),
            None => {
                let idx = self
                    .archetypes
                    .iter()
                    .position(|storage| storage.get_archetype() == archetype);

                if let Some(idx) = idx {
                    self.bundle_cache.insert(archetype, idx);
                    Some(ArchetypeId(idx))
                } else {
                    None
                }
            }
        }
    }

    pub fn get_bundle_archetype(&mut self, bundle: &ComponentBundle) -> ArchetypeId {
        let archetype = bundle
            .iter()
            .map(|component| component.component_id())
            .collect();

        match self.bundle_cache.get(&archetype) {
            Some(idx) => ArchetypeId(*idx),
            None => {
                let idx = self.archetypes.len();
                let storage = EntityStorage::from_bundle(bundle);

                self.archetypes.push(storage);

                for (_type_id, cache) in self.query_cache.borrow_mut().iter_mut() {
                    if cache.pattern.filter(&archetype) {
                        cache.archetypes.push(idx);
                    }
                }

                self.bundle_cache.insert(archetype, idx);
                ArchetypeId(idx)
            }
        }
    }

    pub fn query_archetypes<'a, T: Queryable<'a>>(&'a self) -> Iter<'a> {
        let mut cache = self.query_cache.borrow_mut();

        let type_id = TypeId::of::<(T::Fetch, T::Filter)>();

        let indices = match cache.get(&type_id) {
            Some(pattern) => pattern.archetypes.clone(),
            None => {
                let pattern = T::get_pattern();

                let indices: Vec<_> = self
                    .archetypes
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, storage)| {
                        if pattern.filter(&storage.get_archetype()) {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect();

                cache.insert(
                    type_id,
                    QueryCache {
                        pattern,
                        archetypes: indices.clone(),
                    },
                );

                indices
            }
        };

        Iter::new(&self.archetypes, indices)
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
