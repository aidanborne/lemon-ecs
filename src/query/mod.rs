use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use crate::{
    changes::{ChangeRecord, ChangeStatus},
    collections::sparse_set::{self, SparseSet},
    component::Component,
    entities::EntityId,
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
