use std::{iter::Peekable, marker::PhantomData};

use crate::{
    component::{changes::ChangeRecord, Component},
    storage::{
        archetypes::{ArchetypeIdx, Archetypes},
        entities::EntitySparseSet,
        sparse_set,
    },
    world::World,
};

use self::{
    fetch::QueryFetch,
    filter::{FilterKind, QueryFilter},
};

pub mod fetch;
pub mod filter;

pub struct Query<'world, Fetch: QueryFetch, Filter: QueryFilter = ()> {
    archetypes: &'world Archetypes,
    archetype_idx: Peekable<std::vec::IntoIter<ArchetypeIdx>>,
    entities: Option<sparse_set::Iter<'world, usize>>,
    _fetch: PhantomData<Fetch>,
    _filter: PhantomData<Filter>,
}

impl<'world, Fetch: QueryFetch, Filter: QueryFilter> Query<'world, Fetch, Filter> {
    pub fn new(archetypes: &'world Archetypes, indices: Vec<ArchetypeIdx>) -> Self {
        Self {
            archetypes,
            archetype_idx: indices.into_iter().peekable(),
            entities: None,
            _fetch: PhantomData,
            _filter: PhantomData,
        }
    }

    fn peek_archetype(&mut self) -> Option<&'world EntitySparseSet> {
        self.archetype_idx.peek().map(|idx| &self.archetypes[*idx])
    }

    fn next_entity(&mut self) -> Option<&'world (usize, usize)> {
        loop {
            if let Some(entities) = &mut self.entities {
                if let Some(entity) = entities.next() {
                    return Some(entity);
                }

                self.archetype_idx.next();
            }

            if let Some(archetype) = self.peek_archetype() {
                self.entities = Some(archetype.entities());
            } else {
                return None;
            }
        }
    }

    pub fn get_filters() -> Vec<FilterKind> {
        Fetch::type_ids()
            .into_iter()
            .map(FilterKind::With)
            .chain(Filter::get_filters().into_iter())
            .collect()
    }
}

impl<'world, Fetch: QueryFetch, Filter: QueryFilter> Iterator for Query<'world, Fetch, Filter> {
    type Item = Fetch::Result<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entity()
            .map(|entity| Fetch::fetch(*entity, self.peek_archetype().unwrap()))
    }
}

pub struct ChangeResult<'world, T: Component> {
    world: &'world World,
    record: &'world ChangeRecord,
    id: usize,
    _marker: PhantomData<T>,
}

impl<'world, T: Component> ChangeResult<'world, T> {
    pub fn new(world: &'world World, id: usize, record: &'world ChangeRecord) -> Self {
        Self {
            world,
            record,
            id,
            _marker: PhantomData,
        }
    }

    pub fn added(&self) -> Option<&'world T> {
        if self.record.was_added() {
            self.world.get_component(self.id)
        } else {
            None
        }
    }

    pub fn removed(&self) -> Option<&'world T> {
        self.record
            .get_removed()
            .and_then(|removed| removed.as_any().downcast_ref::<T>())
    }

    pub fn id(&self) -> usize {
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
    type Item = ChangeResult<'world, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(id, record)| ChangeResult::new(self.world, *id, record))
    }
}
