use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Index, IndexMut},
};

use crate::query::{archetype::Archetype, pattern::QueryPattern};

use super::{bundle::ComponentBundle, entities::EntityStorage};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeId(usize);

pub struct QueryResult {
    pub(crate) pattern: QueryPattern,
    pub(crate) archetypes: Vec<ArchetypeId>,
}

#[derive(Default)]
pub struct ArchetypeArena {
    archetypes: Vec<EntityStorage>,
    bundle_cache: HashMap<Archetype, usize>,
    query_cache: HashMap<TypeId, QueryResult>,
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

                for (_type_id, cache) in self.query_cache.iter_mut() {
                    if cache.pattern.filter(&archetype) {
                        cache.archetypes.push(ArchetypeId(idx));
                    }
                }

                self.bundle_cache.insert(archetype, idx);
                ArchetypeId(idx)
            }
        }
    }

    /// Returns the archetypes that matche the given pattern.
    /// Does not cache the result. Caching should be done manually.
    pub fn query_uncached(&self, pattern: QueryPattern) -> QueryResult {
        let archetypes = self
            .archetypes
            .iter()
            .enumerate()
            .filter_map(|(idx, storage)| {
                if pattern.filter(&storage.get_archetype()) {
                    Some(ArchetypeId(idx))
                } else {
                    None
                }
            })
            .collect();

        QueryResult {
            pattern,
            archetypes,
        }
    }

    /// Returns the archetypes that match the given type.
    pub fn query_cached(&self, type_id: TypeId) -> Option<&QueryResult> {
        self.query_cache.get(&type_id)
    }

    pub fn cache_query(&mut self, type_id: TypeId, result: QueryResult) {
        self.query_cache.insert(type_id, result);
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
    archetypes: &'a ArchetypeArena,
    ids: std::vec::IntoIter<ArchetypeId>,
}

impl<'a> Iter<'a> {
    pub fn new(archetypes: &'a ArchetypeArena, ids: Vec<ArchetypeId>) -> Self {
        Self {
            archetypes,
            ids: ids.into_iter(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a EntityStorage;

    fn next(&mut self) -> Option<Self::Item> {
        self.ids.next().map(|idx| &self.archetypes[idx])
    }
}
