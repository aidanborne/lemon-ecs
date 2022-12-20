use std::{any::TypeId, marker::PhantomData};

use lemon_ecs_macros::all_tuples;

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum Filter {
    With(TypeId),
    Without(TypeId),
}

trait FilterGenerator {
    fn get_filter() -> Filter;
}

pub struct With<T>(PhantomData<T>);

impl<T: 'static> FilterGenerator for With<T> {
    fn get_filter() -> Filter {
        Filter::With(TypeId::of::<T>())
    }
}

pub struct Without<T>(PhantomData<T>);

impl<T: 'static> FilterGenerator for Without<T> {
    fn get_filter() -> Filter {
        Filter::Without(TypeId::of::<T>())
    }
}

pub trait QueryFilter {
    fn get_filters() -> Vec<Filter>;
}

impl<T: FilterGenerator> QueryFilter for T {
    fn get_filters() -> Vec<Filter> {
        vec![T::get_filter()]
    }
}

macro_rules! impl_query_filter {
  ($($t:ident),*) => {
      impl<$($t: FilterGenerator),*> QueryFilter for ($($t,)*) {
          fn get_filters() -> Vec<Filter> {
              vec![$($t::get_filter()),*]
          }
      }
  };
}

all_tuples!(impl_query_filter, 0..16);
