use crate::{
    component::Component,
    storage::entities::{Entity, EntityStorage},
};
use lemon_ecs_macros::all_tuples;
use std::any::TypeId;

pub trait QueryFetch {
    type Result<'a>;

    fn get_type_ids() -> Vec<TypeId>;

    fn get_result<'a>(entity: Entity, storage: &'a EntityStorage) -> Self::Result<'a>;
}

impl<T: 'static + Component> QueryFetch for T {
    type Result<'a> = &'a T;

    fn get_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn get_result<'a>(entity: Entity, storage: &'a EntityStorage) -> Self::Result<'a> {
        storage.get_component::<T>(entity.idx()).unwrap()
    }
}

impl QueryFetch for usize {
    type Result<'a> = usize;

    fn get_type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn get_result<'a>(entity: Entity, _storage: &'a EntityStorage) -> Self::Result<'a> {
        entity.id()
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<$($t: 'static + QueryFetch),*> QueryFetch for ($($t,)*) {
            type Result<'a> = ($($t::Result<'a>,)*);

            fn get_type_ids() -> Vec<TypeId> {
                vec![$($t::get_type_ids()),*].concat()
            }

            fn get_result<'a>(entity: Entity, storage: &'a EntityStorage) -> Self::Result<'a> {
                ($($t::get_result(entity, storage),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);
