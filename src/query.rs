use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use lemon_ecs_macros::{for_tuples, impl_query};

use crate::{
    component::Component,
    world::{World, EntityIter}, storage::entities::EntityStorage,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Query(Vec<TypeId>);

#[derive(PartialEq, Clone, Copy)]
pub enum QueryComparison {
    Exact,
    Partial,
    None,
}

impl QueryComparison {
    pub fn is_exact(&self) -> bool {
        matches!(self, QueryComparison::Exact)
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, QueryComparison::None)
    }
}

impl Query {
    pub fn new(type_ids: Vec<TypeId>) -> Self {
        Self(type_ids)
    }

    pub fn normalize(&self) -> Self {
        let mut type_ids = self.0.clone();
        type_ids.sort();
        type_ids.dedup();
        Self::new(type_ids)
    }

    pub fn compare_to(&self, other: &Query) -> QueryComparison {
        let mut set: HashSet<&TypeId> = other.0.iter().collect();

        for type_id in &self.0 {
            if !set.contains(type_id) {
                return QueryComparison::None;
            }

            set.remove(type_id);
        }

        if set.is_empty() {
            QueryComparison::Exact
        } else {
            QueryComparison::Partial
        }
    }
}

pub struct QueryIter<'a, T: Queryable<'a>> {
    entities: EntityIter<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T: Queryable<'a>> QueryIter<'a, T> {
    pub fn new(world: &'a World) -> Self {
        Self {
            entities: world.iter(T::get_query()),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Queryable<'a>> Iterator for QueryIter<'a, T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next().map(|id| T::map_entity(self.entities.get_archetype().unwrap(), id))
    }
}

pub trait Queryable<'a> {
    type Item;

    fn get_query() -> Query;

    fn map_entity(archetype: &'a EntityStorage, id: usize) -> Self::Item;
}

impl<'a, T: 'static + Component> Queryable<'a> for T {
    type Item = &'a T;

    fn get_query() -> Query {
        Query::new(vec![TypeId::of::<T>()])
    }

    fn map_entity(archetype: &'a EntityStorage, id: usize) -> Self::Item {
        archetype.get_component::<T>(id).unwrap()
    }
}

impl <'a> Queryable<'a> for usize {
    type Item = usize;

    fn get_query() -> Query {
        Query::new(vec![])
    }

    fn map_entity(_: &'a EntityStorage, id: usize) -> Self::Item {
        id
    }
}

for_tuples!(impl_query, 2, 16);