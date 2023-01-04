use crate::{component::Component, storage::entities::EntitySparseSet};
use lemon_ecs_macros::all_tuples;
use std::any::TypeId;

pub trait QueryFetch {
    type Result<'world>;

    fn type_ids() -> Vec<TypeId>;

    fn fetch(entity: (usize, usize), storage: &EntitySparseSet) -> Self::Result<'_>;
}

impl<T: 'static + Component> QueryFetch for T {
    type Result<'world> = &'world T;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch((_, storage_idx): (usize, usize), storage: &EntitySparseSet) -> Self::Result<'_> {
        storage.get_component::<T>(storage_idx).unwrap()
    }
}

impl QueryFetch for usize {
    type Result<'world> = usize;

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn fetch((id, _): (usize, usize), _storage: &EntitySparseSet) -> Self::Result<'_> {
        id
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<$($t: 'static + QueryFetch),*> QueryFetch for ($($t,)*) {
            type Result<'world> = ($($t::Result<'world>,)*);

            fn type_ids() -> Vec<TypeId> {
                vec![$($t::type_ids()),*].concat()
            }

            fn fetch(entity: (usize, usize), storage: &EntitySparseSet) -> Self::Result<'_> {
                ($($t::fetch(entity, storage),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);
