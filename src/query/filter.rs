use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use lemon_ecs_macros::all_tuples;

#[derive(PartialEq, Eq, Hash)]
pub enum FilterKind {
    With(TypeId),
    Without(TypeId),
}

pub trait Filter {
    fn filter(&self, type_ids: &HashSet<TypeId>) -> bool;
}

impl Filter for FilterKind {
    fn filter(&self, type_ids: &HashSet<TypeId>) -> bool {
        match self {
            FilterKind::With(type_id) => type_ids.contains(type_id),
            FilterKind::Without(type_id) => !type_ids.contains(type_id),
        }
    }
}

impl Filter for Vec<FilterKind> {
    fn filter(&self, type_ids: &HashSet<TypeId>) -> bool {
        for filter in self.iter() {
            if !filter.filter(type_ids) {
                return false;
            }
        }

        true
    }
}

trait Filterable {
    fn get_filter() -> FilterKind;
}

pub struct With<T>(PhantomData<T>);

impl<T: 'static> Filterable for With<T> {
    fn get_filter() -> FilterKind {
        FilterKind::With(TypeId::of::<T>())
    }
}

pub struct Without<T>(PhantomData<T>);

impl<T: 'static> Filterable for Without<T> {
    fn get_filter() -> FilterKind {
        FilterKind::Without(TypeId::of::<T>())
    }
}

pub trait QueryFilter {
    fn get_filters() -> Vec<FilterKind>;
}

impl<T: Filterable> QueryFilter for T {
    fn get_filters() -> Vec<FilterKind> {
        vec![T::get_filter()]
    }
}

macro_rules! impl_query_filter {
  ($($t:ident),*) => {
      impl<$($t: Filterable),*> QueryFilter for ($($t,)*) {
          fn get_filters() -> Vec<FilterKind> {
              vec![$($t::get_filter()),*]
          }
      }
  };
}

all_tuples!(impl_query_filter, 0..16);
