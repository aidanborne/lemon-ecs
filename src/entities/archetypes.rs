use std::{any::TypeId, collections::HashMap};

use crate::{component::Component, query::QuerySelector};

use super::{Archetype, EntityId, EntityIter, IdIter, Indices};

struct QueryResult {
    filter: fn(&Archetype) -> bool,
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

        let idx = self.bundle_cache.entry(type_ids).or_insert_with(|| {
            let idx = self.archetypes.len();

            self.archetypes.push(Archetype::from_components(components));
            let archetype = &self.archetypes[idx];

            for cache in self.query_cache.values_mut() {
                if (cache.filter)(&archetype) {
                    cache.indices.push(idx);
                }
            }

            idx
        });

        &mut self.archetypes[*idx]
    }

    fn query_indices<T>(&mut self) -> Indices<Archetype>
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
                        if T::filter(archetype) {
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

        Indices::new(&self.archetypes, &result.indices)
    }

    pub fn query_entities<T>(&mut self) -> EntityIter
    where
        T: 'static + QuerySelector,
    {
        self.query_indices::<T>().into()
    }

    pub(crate) fn query_ids<T>(&mut self) -> IdIter
    where
        T: 'static + QuerySelector,
    {
        self.query_indices::<T>().into()
    }
}
