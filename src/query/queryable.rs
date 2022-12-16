use std::any::TypeId;

use lemon_ecs_macros::{for_tuples, impl_query};

use crate::{storage::entities::EntityStorage, component::Component};

use super::Query;

pub trait Queryable<'a> {
  type Item;

  fn get_query() -> Query;

  fn map_entity(archetype: &'a EntityStorage, id: usize) -> Self::Item;
}

impl<'a, T: 'static + Component> Queryable<'a> for T {
  type Item = &'a T;

  fn get_query() -> Query {
      Query::new(vec![TypeId::of::<T>()])
  }

  fn map_entity(archetype: &'a EntityStorage, id: usize) -> Self::Item {
      archetype.get_component::<T>(id).unwrap()
  }
}

impl <'a> Queryable<'a> for usize {
  type Item = usize;

  fn get_query() -> Query {
      Query::new(vec![])
  }

  fn map_entity(_: &'a EntityStorage, id: usize) -> Self::Item {
      id
  }
}

for_tuples!(impl_query, 2, 16);