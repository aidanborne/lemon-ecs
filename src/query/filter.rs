use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use lemon_ecs_macros::all_tuples;

pub struct With<T>(PhantomData<T>);

pub struct Without<T>(PhantomData<T>);

pub trait QueryFilter {
    fn filter(type_ids: &HashSet<TypeId>) -> bool;
}

impl<T: 'static> QueryFilter for With<T> {
    fn filter(type_ids: &HashSet<TypeId>) -> bool {
        type_ids.contains(&TypeId::of::<T>())
    }
}

impl<T: 'static> QueryFilter for Without<T> {
    fn filter(type_ids: &HashSet<TypeId>) -> bool {
        !type_ids.contains(&TypeId::of::<T>())
    }
}

macro_rules! impl_query_filter {
  ($($t:ident),*) => {
      impl<$($t: QueryFilter),*> QueryFilter for ($($t,)*) {
          fn filter(_type_ids: &HashSet<TypeId>) -> bool {
                $($t::filter(_type_ids) &&)* true
          }
      }
  };
}

all_tuples!(impl_query_filter, 0..16);
