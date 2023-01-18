use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use crate::{
    changes::{ChangeRecord, ChangeStatus},
    collections::sparse_set,
    component::Component,
    entities::{
        archetype::{self, Archetype, Entity},
        EntityId,
    },
    world::World,
};

mod fetch;
mod filter;

pub use fetch::QueryFetch;
pub use filter::QueryFilter;
pub use filter::{With, Without};
use sparse_set::SparseSet;

pub struct Query<'world, Fetch: QueryFetch, Filter: QueryFilter = ()> {
    world: &'world World,
    archetypes: std::vec::IntoIter<&'world Archetype>,
    entities: Option<archetype::Iter<'world>>,
    _fetch: PhantomData<Fetch>,
    _filter: PhantomData<Filter>,
}

impl<'world, Fetch: QueryFetch, Filter: QueryFilter> Query<'world, Fetch, Filter> {
    pub fn new(world: &'world World, archetypes: std::vec::IntoIter<&'world Archetype>) -> Self {
        Self {
            world,
            archetypes,
            entities: None,
            _fetch: PhantomData,
            _filter: PhantomData,
        }
    }

    fn next_entity(&mut self) -> Option<Entity<'world>> {
        loop {
            if let Some(entities) = &mut self.entities {
                let entity = entities.next();

                if entity.is_some() {
                    return entity;
                }
            }

            if let Some(archetype) = self.archetypes.next() {
                self.entities = Some(archetype.iter());
            } else {
                return None;
            }
        }
    }

    pub fn should_query(type_ids: &HashSet<TypeId>) -> bool {
        Fetch::should_fetch(type_ids) && Filter::filter(type_ids)
    }
}

impl<'world, Fetch: QueryFetch, Filter: QueryFilter> Iterator for Query<'world, Fetch, Filter> {
    type Item = Fetch::Output<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entity()
            .map(|entity| Fetch::fetch(self.world, &entity))
    }
}

enum ComponentChanged<'world, 'query, T: Component> {
    New(&'world World),
    Old(&'query T),
    Both(&'world World, &'query T),
}

pub struct EntityChanged<'world, 'query, T: Component> {
    id: EntityId,
    component: ComponentChanged<'world, 'query, T>,
}

impl<'world, 'query, T: Component> EntityChanged<'world, 'query, T> {
    pub fn get_new(&self) -> Option<&'world T> {
        match &self.component {
            ComponentChanged::New(world) | ComponentChanged::Both(world, _) => {
                world.get_component(self.id)
            }
            _ => None,
        }
    }

    pub fn get_old(&self) -> Option<&T> {
        match &self.component {
            ComponentChanged::Old(old) | ComponentChanged::Both(_, old) => Some(old),
            _ => None,
        }
    }

    #[inline]
    pub fn id(&self) -> EntityId {
        self.id
    }
}

pub struct QueryChanged<'world, T: Component> {
    world: &'world World,
    entities: SparseSet<ChangeStatus>,
    removed: Vec<T>,
}

impl<'world, T: Component> QueryChanged<'world, T> {
    pub(crate) fn new(world: &'world World, record: ChangeRecord) -> Self {
        Self {
            world,
            entities: record.entities,
            removed: *record.removed.downcast().ok().unwrap(),
        }
    }
}

impl<'world, 'query, T: Component> IntoIterator for &'query QueryChanged<'world, T> {
    type Item = EntityChanged<'world, 'query, T>;
    type IntoIter = QueryChangedIter<'world, 'query, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueryChangedIter {
            iter: self.entities.iter(),
            world: self.world,
            removed: &self.removed,
        }
    }
}

pub struct QueryChangedIter<'world, 'query, T: Component> {
    iter: sparse_set::Iter<'query, ChangeStatus>,
    world: &'world World,
    removed: &'query Vec<T>,
}

impl<'world, 'query, T: Component> Iterator for QueryChangedIter<'world, 'query, T> {
    type Item = EntityChanged<'world, 'query, T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let option = self.iter.next();

            if let Some((id, record)) = option {
                return Some(EntityChanged {
                    id: (*id).into(),
                    component: match record {
                        ChangeStatus::Added => ComponentChanged::New(self.world),
                        ChangeStatus::Removed(idx) => ComponentChanged::Old(&self.removed[*idx]),
                        ChangeStatus::Modified(idx) => {
                            ComponentChanged::Both(self.world, &self.removed[*idx])
                        }
                    },
                });
            } else {
                return None;
            }
        }
    }
}
