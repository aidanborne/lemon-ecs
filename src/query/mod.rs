use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use crate::{
    changes::ChangeRecord,
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

pub struct EntityChange<'world, T: Component> {
    world: &'world World,
    record: &'world ChangeRecord,
    id: EntityId,
    _marker: PhantomData<T>,
}

impl<'world, T: Component> EntityChange<'world, T> {
    pub fn new(world: &'world World, id: EntityId, record: &'world ChangeRecord) -> Self {
        Self {
            world,
            record,
            id,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn is_added(&self) -> bool {
        self.record.is_added()
    }

    #[inline]
    pub fn is_changed(&self) -> bool {
        self.record.is_changed()
    }

    #[inline]
    pub fn is_removed(&self) -> bool {
        self.record.is_removed()
    }

    pub fn current(&self) -> Option<&'world T> {
        if self.record.is_added() || self.record.is_changed() {
            self.world.get_component(self.id)
        } else {
            None
        }
    }

    pub fn removed(&self) -> Option<&T> {
        self.record
            .get_removed()
            .and_then(|removed| removed.as_any().downcast_ref::<T>())
    }

    #[inline]
    pub fn id(&self) -> EntityId {
        self.id
    }
}

pub struct QueryChanged<'world, T: Component> {
    world: &'world World,
    iter: sparse_set::Iter<'world, ChangeRecord>,
    _marker: PhantomData<T>,
}

impl<'world, T: Component> QueryChanged<'world, T> {
    pub fn new(world: &'world World, iter: sparse_set::Iter<'world, ChangeRecord>) -> Self {
        Self {
            world,
            iter,
            _marker: PhantomData,
        }
    }
}

impl<'world, T: Component> Iterator for QueryChanged<'world, T> {
    type Item = EntityChange<'world, T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let option = self.iter.next();

            if let Some((id, record)) = option {
                // Ignore no change records, they are not interesting
                if !record.is_no_change() {
                    return Some(EntityChange::new(self.world, (*id).into(), record));
                }
            } else {
                return None;
            }
        }
    }
}
