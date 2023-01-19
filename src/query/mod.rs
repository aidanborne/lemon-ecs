use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use crate::{
    changes::{AddedIter, ChangeRecord, EntitySnapshot, ModifiedIter, RemovedIter, SnapshotIter},
    component::Component,
    world::World,
};

use lemon_ecs_macros::all_tuples;

mod filter;
mod iter;
mod retriever;

pub use filter::Without;
pub use retriever::QueryRetriever;

pub use self::iter::*;

pub trait QuerySelector: 'static {
    /// Returns true if the query should be run for the given archetype.
    fn filter(type_ids: &HashSet<TypeId>) -> bool;
}

macro_rules! impl_tuple_selector {
    ($($t:ident),*) => {
        impl<$($t: QuerySelector),*> QuerySelector for ($($t,)*) {
            fn filter(_type_ids: &HashSet<TypeId>) -> bool {
                $($t::filter(_type_ids) &&)* true
            }
        }
    };
  }

all_tuples!(impl_tuple_selector, 0..16);

pub struct Query<'world, T: QueryRetriever>(&'world mut World, PhantomData<T>);

impl<'world, T: QueryRetriever> Query<'world, T> {
    pub fn new(world: &'world mut World) -> Self {
        Self(world, PhantomData)
    }

    #[inline]
    pub fn filter<Q: QuerySelector>(self) -> QueryIter<'world, T> {
        QueryIter::new(self.0.query_selector::<(T, Q)>())
    }
}

impl<'world, T: QueryRetriever> IntoIterator for Query<'world, T> {
    type Item = T::Output<'world>;
    type IntoIter = QueryIter<'world, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter::new(self.0.query_selector::<T>())
    }
}

pub struct QueryChanged<'world, T: Component> {
    world: &'world World,
    record: ChangeRecord,
    marker: std::marker::PhantomData<T>,
}

impl<'world, T: Component> QueryChanged<'world, T> {
    pub(crate) fn new(world: &'world World, record: ChangeRecord) -> Self {
        Self {
            world,
            record,
            marker: std::marker::PhantomData,
        }
    }

    /// Returns an iterator over all entities that have been added or changed.
    pub fn added(self) -> AddedIter<'world, T> {
        AddedIter::new(self.world, self.record)
    }

    /// Returns an iterator over all entities that have been changed.
    pub fn modified(self) -> ModifiedIter<'world, T> {
        ModifiedIter::new(self.world, self.record)
    }

    /// Returns an iterator over all entities that have been changed or removed.
    pub fn removed(self) -> RemovedIter<T> {
        RemovedIter::new(self.record)
    }
}

impl<'world, T: Component> IntoIterator for QueryChanged<'world, T> {
    type Item = EntitySnapshot<'world, T>;
    type IntoIter = SnapshotIter<'world, T>;

    fn into_iter(self) -> Self::IntoIter {
        SnapshotIter::new(self.world, self.record)
    }
}
