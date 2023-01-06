use lemon_ecs_macros::all_tuples;

use std::{
    any::TypeId,
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use crate::{
    collections::entity_sparse_set::Entity,
    component::{Component, ComponentChange},
    world::{EntityId, World, WorldUpdate},
};

pub trait QueryFetch {
    type Result<'world>;

    fn type_ids() -> Vec<TypeId>;

    fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Result<'world>;
}

impl<T: 'static + Component> QueryFetch for &'_ T {
    type Result<'world> = &'world T;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch<'world>(_world: &'world World, entity: &Entity<'world>) -> Self::Result<'world> {
        entity.get_component::<T>().unwrap()
    }
}

impl QueryFetch for EntityId {
    type Result<'world> = EntityId;

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn fetch<'world>(_world: &'world World, entity: &Entity<'world>) -> Self::Result<'world> {
        entity.id()
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<$($t: QueryFetch),*> QueryFetch for ($($t,)*) {
            type Result<'world> = ($($t::Result<'world>,)*);

            fn type_ids() -> Vec<TypeId> {
                let type_ids: Vec<Vec<TypeId>> = vec![$($t::type_ids()),*];
                type_ids.concat()
            }

            fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Result<'world> {
                ($($t::fetch(world, entity),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);

pub struct ComponentMut<'world, T: Component + Clone> {
    world: &'world World,
    id: EntityId,
    value: Cow<'world, T>,
}

impl<'world, T: Component + Clone> ComponentMut<'world, T> {
    pub fn new(world: &'world World, id: EntityId, value: &'world T) -> Self {
        Self {
            world,
            id,
            value: Cow::Borrowed(value),
        }
    }
}

impl<T: Component + Clone> Deref for ComponentMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<T: Component + Clone> DerefMut for ComponentMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.to_mut()
    }
}

impl<T: Component + Clone> Drop for ComponentMut<'_, T> {
    fn drop(&mut self) {
        if let Cow::Owned(value) = &self.value {
            let change = ComponentChange::Added(Box::new(value.clone()));

            self.world
                .push_update(WorldUpdate::ModifyEntity(self.id, vec![change]))
        }
    }
}

impl<T: Component + Clone> QueryFetch for ComponentMut<'_, T> {
    type Result<'world> = ComponentMut<'world, T>;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Result<'world> {
        ComponentMut::new(world, entity.id(), entity.get_component::<T>().unwrap())
    }
}

impl<T: Component + Clone> QueryFetch for &'_ mut T {
    type Result<'world> = ComponentMut<'world, T>;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Result<'world> {
        ComponentMut::new(world, entity.id(), entity.get_component::<T>().unwrap())
    }
}
