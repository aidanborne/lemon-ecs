use crate::{
    component::Component,
    storage::entities::{Entity, EntityStorage},
};
use lemon_ecs_macros::all_tuples;
use std::any::TypeId;

pub trait QueryFetch<'a> {
    type Result;

    fn get_type_ids() -> Vec<TypeId>;

    fn get_result(entity: Entity, storage: &'a EntityStorage) -> Self::Result;
}

impl<'a, T: 'static + Component> QueryFetch<'a> for T {
    type Result = &'a T;

    fn get_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn get_result(entity: Entity, storage: &'a EntityStorage) -> Self::Result {
        storage.get_component::<T>(entity.idx()).unwrap()
    }
}

impl QueryFetch<'_> for usize {
    type Result = usize;

    fn get_type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn get_result(entity: Entity, _storage: &EntityStorage) -> Self::Result {
        entity.id()
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<'a, $($t: 'static + Component + QueryFetch<'a>),*> QueryFetch<'a> for ($($t,)*) {
            type Result = ($($t::Result,)*);

            fn get_type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$t>()),*]
            }

            fn get_result(entity: Entity, storage: &'a EntityStorage) -> Self::Result {
                ($($t::get_result(entity, storage),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);
