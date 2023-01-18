use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use crate::{component::Component, query::QuerySelector};

use super::{archetype::ArchetypeIter, Archetype, Entity, EntityId};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct ArchetypeIdx(usize);

struct QueryResult {
    filter: fn(&HashSet<TypeId>) -> bool,
    indices: Vec<usize>,
}

#[derive(Default)]
pub(crate) struct Archetypes {
    archetypes: Vec<Archetype>,
    bundle_cache: HashMap<Vec<TypeId>, usize>,
    query_cache: HashMap<TypeId, QueryResult>,
}

impl Archetypes {
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
        let type_ids: Vec<TypeId> = components
            .iter()
            .map(|component| component.as_any().type_id())
            .collect();

        let hash_set: HashSet<TypeId> = type_ids.iter().cloned().collect();

        let idx = self.bundle_cache.entry(type_ids).or_insert_with(|| {
            let idx = self.archetypes.len();

            let archetype = Archetype::from_components(components);
            self.archetypes.push(archetype);

            for cache in self.query_cache.values_mut() {
                if (cache.filter)(&hash_set) {
                    cache.indices.push(idx);
                }
            }

            idx
        });

        &mut self.archetypes[*idx]
    }

    pub fn query_entities<T>(&mut self) -> EntityIter
    where
        T: 'static + QuerySelector,
    {
        let result = self
            .query_cache
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                let indices = self
                    .archetypes
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, archetype)| {
                        if T::filter(&archetype.type_ids()) {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect();

                QueryResult {
                    filter: T::filter,
                    indices,
                }
            });

        EntityIter {
            archetypes: &self.archetypes,
            indices: result.indices.iter(),
            iter: None,
        }
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

pub struct EntityIter<'archetype> {
    archetypes: &'archetype [Archetype],
    indices: std::slice::Iter<'archetype, usize>,
    iter: Option<ArchetypeIter<'archetype>>,
}

impl<'archetype> Iterator for EntityIter<'archetype> {
    type Item = Entity<'archetype>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.as_mut() {
                Some(iter) => match iter.next() {
                    Some(entity) => return Some(entity),
                    None => self.iter = None,
                },
                None => match self.indices.next() {
                    Some(idx) => self.iter = Some(self.archetypes[*idx].iter()),
                    None => return None,
                },
            }
        }
    }
}
