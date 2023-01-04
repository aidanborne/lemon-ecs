use crate::{component::Component, storage::entities::EntityStorage};
use lemon_ecs_macros::all_tuples;
use std::any::TypeId;

pub trait QueryFetch {
    type Result<'a>;

    fn get_type_ids() -> Vec<TypeId>;

    fn get_result<'world>(
        entity: (usize, usize),
        storage: &'world EntityStorage,
    ) -> Self::Result<'world>;
}

impl<T: 'static + Component> QueryFetch for T {
    type Result<'a> = &'a T;

    fn get_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn get_result<'a>(
        (_, storage_idx): (usize, usize),
        storage: &'a EntityStorage,
    ) -> Self::Result<'a> {
        storage.get_component::<T>(storage_idx).unwrap()
    }
}

impl QueryFetch for usize {
    type Result<'a> = usize;

    fn get_type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn get_result<'a>((id, _): (usize, usize), _storage: &'a EntityStorage) -> Self::Result<'a> {
        id
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<$($t: 'static + QueryFetch),*> QueryFetch for ($($t,)*) {
            type Result<'a> = ($($t::Result<'a>,)*);

            fn get_type_ids() -> Vec<TypeId> {
                vec![$($t::get_type_ids()),*].concat()
            }

            fn get_result<'a>(entity: (usize, usize), storage: &'a EntityStorage) -> Self::Result<'a> {
                ($($t::get_result(entity, storage),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);
