use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use crate::{
    component::Component,
    query::{Query, QueryFetch, QueryFilter},
};

use super::{Archetype, EntityId};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct ArchetypeIdx(usize);

struct QueryResult {
    filter: fn(&HashSet<TypeId>) -> bool,
    indices: Vec<usize>,
}

pub(crate) struct Archetypes {
    archetypes: Vec<Archetype>,
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

    pub fn entity_archetype(&self, id: EntityId) -> Option<&Archetype> {
        self.archetypes
            .iter()
            .find(|archetype| archetype.contains(id))
    }

    pub fn entity_archetype_mut(&mut self, id: EntityId) -> Option<&mut Archetype> {
        self.archetypes
            .iter_mut()
            .find(|archetype| archetype.contains(id))
    }

    pub fn component_archetype(&mut self, components: &[Box<dyn Component>]) -> &mut Archetype {
        let archetype: Vec<TypeId> = components
            .iter()
            .map(|component| component.as_any().type_id())
            .collect();

        let hash_set: HashSet<TypeId> = archetype.iter().cloned().collect();

        match self.bundle_cache.get(&archetype) {
            Some(idx) => &mut self.archetypes[*idx],
            None => {
                let idx = self.archetypes.len();
                let storage = Archetype::from_components(components);

                self.archetypes.push(storage);

                for (_type_id, cache) in self.query_cache.borrow_mut().iter_mut() {
                    if (cache.filter)(&hash_set) {
                        cache.indices.push(idx);
                    }
                }

                self.bundle_cache.insert(archetype, idx);
                self.archetypes.last_mut().unwrap()
            }
        }
    }

    pub fn query_archetypes<Fetch, Filter>(&self) -> std::vec::IntoIter<&Archetype>
    where
        Fetch: 'static + QueryFetch,
        Filter: 'static + QueryFilter,
    {
        let mut query_cache = self.query_cache.borrow_mut();
        let type_id = TypeId::of::<(Fetch, Filter)>();

        let filter = Query::<Fetch, Filter>::should_query;

        let indices = match query_cache.get(&type_id) {
            Some(result) => &result.indices,
            None => {
                let indices = self
                    .archetypes
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, archetype)| {
                        if filter(&archetype.type_ids()) {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect();

                let result = QueryResult { filter, indices };

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
    type Output = Archetype;

    fn index(&self, idx: ArchetypeIdx) -> &Self::Output {
        &self.archetypes[idx.0]
    }
}

impl IndexMut<ArchetypeIdx> for Archetypes {
    fn index_mut(&mut self, index: ArchetypeIdx) -> &mut Self::Output {
        &mut self.archetypes[index.0]
    }
}