use lemon_ecs_macros::all_tuples;

use std::{any::TypeId, collections::HashSet};

use crate::{
    component::Component,
    entities::{Entity, EntityId},
};

use super::QuerySelector;

/// Allows fetching components from an entity.
pub trait QueryRetriever: QuerySelector {
    type Output<'world>;

    /// Retrieves the component(s) from the given entity.
    /// This function should never panic.
    fn retrieve<'world>(entity: &Entity<'world>) -> Self::Output<'world>;
}

impl<T: 'static + Component> QueryRetriever for T {
    type Output<'world> = &'world T;

    fn retrieve<'world>(entity: &Entity<'world>) -> Self::Output<'world> {
        entity.get_component::<T>().unwrap()
    }
}

impl<'a, T: 'static + Component> QuerySelector for T {
    #[inline]
    fn filter(type_ids: &HashSet<TypeId>) -> bool {
        type_ids.contains(&TypeId::of::<T>())
    }
}

impl QueryRetriever for EntityId {
    type Output<'world> = EntityId;

    fn retrieve<'world>(entity: &Entity<'world>) -> Self::Output<'world> {
        entity.id()
    }
}

impl QuerySelector for EntityId {
    #[inline]
    fn filter(_type_ids: &HashSet<TypeId>) -> bool {
        true
    }
}

macro_rules! impl_tuple_retriever {
    ($($t:ident),*) => {
        impl<$($t: QueryRetriever),*> QueryRetriever for ($($t,)*) where Self: QuerySelector {
            type Output<'world> = ($($t::Output<'world>,)*);

            fn retrieve<'world>(_entity: &Entity<'world>) -> Self::Output<'world> {
                ($($t::retrieve(_entity),)*)
            }
        }
    };
  }

all_tuples!(impl_tuple_retriever, 0..16);
