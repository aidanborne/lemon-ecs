use crate::{component::Component, storage::entities::Entity, world::entities::EntityId};
use lemon_ecs_macros::all_tuples;
use std::any::TypeId;

pub trait QueryFetch {
    type Result<'world>;

    fn type_ids() -> Vec<TypeId>;

    fn fetch<'world>(entity: &Entity<'world>) -> Self::Result<'world>;
}

impl<T: 'static + Component> QueryFetch for T {
    type Result<'world> = &'world T;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch<'world>(entity: &Entity<'world>) -> Self::Result<'world> {
        entity.get_component::<T>().unwrap()
    }
}

impl QueryFetch for EntityId {
    type Result<'world> = EntityId;

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn fetch<'world>(entity: &Entity<'world>) -> Self::Result<'world> {
        entity.id()
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<$($t: 'static + QueryFetch),*> QueryFetch for ($($t,)*) {
            type Result<'world> = ($($t::Result<'world>,)*);

            fn type_ids() -> Vec<TypeId> {
                vec![$($t::type_ids()),*].concat()
            }

            fn fetch<'world>(entity: &Entity<'world>) -> Self::Result<'world> {
                ($($t::fetch(entity),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);
