use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use lemon_ecs_macros::all_tuples;

#[derive(PartialEq, Eq, Hash)]
pub enum Filter {
    With(TypeId),
    Without(TypeId),
}

impl Filter {
    pub fn filter(&self, type_ids: &HashSet<TypeId>) -> bool {
        match self {
            Filter::With(type_id) => type_ids.contains(type_id),
            Filter::Without(type_id) => !type_ids.contains(type_id),
        }
    }

    pub fn filter_all(filters: &[Filter], type_ids: &HashSet<TypeId>) -> bool {
        filters.iter().all(|filter| filter.filter(type_ids))
    }
}

trait Filterable {
    fn get_filter() -> Filter;
}

pub struct With<T>(PhantomData<T>);

impl<T: 'static> Filterable for With<T> {
    fn get_filter() -> Filter {
        Filter::With(TypeId::of::<T>())
    }
}

pub struct Without<T>(PhantomData<T>);

impl<T: 'static> Filterable for Without<T> {
    fn get_filter() -> Filter {
        Filter::Without(TypeId::of::<T>())
    }
}

pub trait QueryFilter {
    fn get_filters() -> Vec<Filter>;
}

impl<T: Filterable> QueryFilter for T {
    fn get_filters() -> Vec<Filter> {
        vec![T::get_filter()]
    }
}

macro_rules! impl_query_filter {
  ($($t:ident),*) => {
      impl<$($t: Filterable),*> QueryFilter for ($($t,)*) {
          fn get_filters() -> Vec<Filter> {
              vec![$($t::get_filter()),*]
          }
      }
  };
}

all_tuples!(impl_query_filter, 0..16);
