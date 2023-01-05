use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use crate::{
    component::Component,
    query::{
        fetch::QueryFetch,
        filter::{Filter, FilterKind, QueryFilter},
        Query,
    },
    world::entities::EntityId,
};

use super::entities::EntitySparseSet;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeIdx(usize);

struct QueryResult {
    filter_kinds: Vec<FilterKind>,
    indices: Vec<usize>,
}

pub struct Archetypes {
    archetypes: Vec<EntitySparseSet>,
    bundle_cache: HashMap<Vec<TypeId>, usize>,
    query_cache: RefCell<HashMap<TypeId, QueryResult>>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            bundle_cache: HashMap::new(),
            query_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn entity_archetype_idx(&self, id: EntityId) -> Option<ArchetypeIdx> {
        self.archetypes
            .iter()
            .position(|archetype| archetype.contains(id))
            .map(ArchetypeIdx)
    }

    pub fn entity_archetype(&self, id: EntityId) -> Option<&EntitySparseSet> {
        self.archetypes
            .iter()
            .find(|archetype| archetype.contains(id))
    }

    pub fn entity_archetype_mut(&mut self, id: EntityId) -> Option<&mut EntitySparseSet> {
        self.archetypes
            .iter_mut()
            .find(|archetype| archetype.contains(id))
    }

    pub fn component_archetype(
        &mut self,
        components: &[Box<dyn Component>],
    ) -> &mut EntitySparseSet {
        let archetype: Vec<TypeId> = components
            .iter()
            .map(|component| component.as_any().type_id())
            .collect();

        let hash_set: HashSet<TypeId> = archetype.iter().cloned().collect();

        match self.bundle_cache.get(&archetype) {
            Some(idx) => &mut self.archetypes[*idx],
            None => {
                let idx = self.archetypes.len();
                let storage = EntitySparseSet::from_components(components);

                self.archetypes.push(storage);

                for (_type_id, cache) in self.query_cache.borrow_mut().iter_mut() {
                    if cache.filter_kinds.filter(&hash_set) {
                        cache.indices.push(idx);
                    }
                }

                self.bundle_cache.insert(archetype, idx);
                self.archetypes.last_mut().unwrap()
            }
        }
    }

    pub fn query_archetypes<Fetch, Filter>(&self) -> std::vec::IntoIter<&EntitySparseSet>
    where
        Fetch: 'static + QueryFetch,
        Filter: 'static + QueryFilter,
    {
        let mut query_cache = self.query_cache.borrow_mut();
        let type_id = TypeId::of::<(Fetch, Filter)>();

        let indices = match query_cache.get(&type_id) {
            Some(result) => &result.indices,
            None => {
                let filter_kinds = Query::<Fetch, Filter>::get_filters();

                let indices = self
                    .archetypes
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, archetype)| {
                        if filter_kinds.filter(&archetype.type_ids()) {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect();

                let result = QueryResult {
                    filter_kinds,
                    indices,
                };

                query_cache.insert(type_id, result);
                &query_cache.get(&type_id).unwrap().indices
            }
        };

        indices
            .iter()
            .map(|idx| &self.archetypes[*idx])
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Default for Archetypes {
    fn default() -> Self {
        Self::new()
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
