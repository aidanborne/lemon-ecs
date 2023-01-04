use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use crate::{
    component::bundle::ComponentBundle,
    query::filter::{Filter, FilterKind},
};

use super::entities::EntitySparseSet;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeIdx(usize);

pub struct QueryResult {
    filter_kinds: Vec<FilterKind>,
    indices: Vec<ArchetypeIdx>,
}

impl QueryResult {
    pub fn indices(&self) -> Vec<ArchetypeIdx> {
        self.indices.clone()
    }
}

#[derive(Default)]
pub struct Archetypes {
    archetypes: Vec<EntitySparseSet>,
    bundle_cache: HashMap<Vec<TypeId>, usize>,
    query_cache: HashMap<TypeId, QueryResult>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_entity_archetype(&self, id: usize) -> Option<ArchetypeIdx> {
        self.archetypes
            .iter()
            .position(move |archetype| archetype.contains(id))
            .map(|idx| ArchetypeIdx(idx))
    }

    pub fn get_existing_archetype(&mut self, type_ids: Vec<TypeId>) -> Option<ArchetypeIdx> {
        match self.bundle_cache.get(&type_ids) {
            Some(idx) => Some(ArchetypeIdx(*idx)),
            None => {
                let idx = self
                    .archetypes
                    .iter()
                    .position(|storage| storage.type_ids::<Vec<TypeId>>() == type_ids);

                if let Some(idx) = idx {
                    self.bundle_cache.insert(type_ids, idx);
                    Some(ArchetypeIdx(idx))
                } else {
                    None
                }
            }
        }
    }

    pub fn get_bundle_archetype(&mut self, bundle: &ComponentBundle) -> ArchetypeIdx {
        let archetype: Vec<TypeId> = bundle
            .iter()
            .map(|component| component.as_any().type_id())
            .collect();

        let hash_set: HashSet<TypeId> = archetype.iter().cloned().collect();

        match self.bundle_cache.get(&archetype) {
            Some(idx) => ArchetypeIdx(*idx),
            None => {
                let idx = self.archetypes.len();
                let storage = EntitySparseSet::from_bundle(bundle);

                self.archetypes.push(storage);

                for (_type_id, cache) in self.query_cache.iter_mut() {
                    if cache.filter_kinds.filter(&hash_set) {
                        cache.indices.push(ArchetypeIdx(idx));
                    }
                }

                self.bundle_cache.insert(archetype, idx);
                ArchetypeIdx(idx)
            }
        }
    }

    /// Returns the archetypes that matche the given pattern.
    /// Does not cache the result. Caching should be done manually.
    pub fn query_uncached(&self, filter_kinds: Vec<FilterKind>) -> QueryResult {
        let indices = self
            .archetypes
            .iter()
            .enumerate()
            .filter_map(|(idx, storage)| {
                if filter_kinds.filter(&storage.type_ids()) {
                    Some(ArchetypeIdx(idx))
                } else {
                    None
                }
            })
            .collect();

        QueryResult {
            filter_kinds,
            indices,
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

impl Index<ArchetypeIdx> for Archetypes {
    type Output = EntitySparseSet;

    fn index(&self, idx: ArchetypeIdx) -> &Self::Output {
        &self.archetypes[idx.0]
    }
}

impl IndexMut<ArchetypeIdx> for Archetypes {
    fn index_mut(&mut self, index: ArchetypeIdx) -> &mut Self::Output {
        &mut self.archetypes[index.0]
    }
}
