use crate::{component::Component, storage::entities::EntityStorage};
use lemon_ecs_macros::for_tuples;
use std::any::TypeId;

use super::archetype::Archetype;

pub trait Queryable<'a> {
    type Result;

    fn get_archetype() -> Archetype;

    fn map_entity(archetype: &'a EntityStorage, id: usize) -> Self::Result;
}

impl<'a, T: 'static + Component> Queryable<'a> for T {
    type Result = &'a T;

    fn get_archetype() -> Archetype {
        Archetype::new(vec![TypeId::of::<T>()])
    }

    fn map_entity(storage: &'a EntityStorage, id: usize) -> Self::Result {
        storage.get_component::<T>(id).unwrap()
    }
}

impl<'a> Queryable<'a> for usize {
    type Result = usize;

    fn get_archetype() -> Archetype {
        Archetype::new(vec![])
    }

    fn map_entity(_: &EntityStorage, id: usize) -> Self::Result {
        id
    }
}

macro_rules! impl_query {
    ($($t:ident),*) => {
        impl<'a, $($t: 'static + $crate::query::Queryable<'a>),*> $crate::query::Queryable<'a> for ($($t,)*) {
            type Result = ($($t::Result,)*);

            fn get_archetype() -> Archetype {
                Archetype::new(vec![$(TypeId::of::<$t>()),*])
            }

            fn map_entity(storage: &'a EntityStorage, id: usize) -> Self::Result {
                ($($t::map_entity(storage, id)),*)
            }
        }
    };
  }

for_tuples!(impl_query, 2, 16);
