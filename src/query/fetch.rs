use lemon_ecs_macros::all_tuples;

use std::{
    any::TypeId,
    borrow::Cow,
    collections::HashSet,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use crate::{
    component::{Component, ComponentChange},
    entities::{Entity, EntityId},
    world::{World, WorldUpdate},
};

pub trait QueryFetch {
    type Output<'world>;

    fn should_fetch(type_ids: &HashSet<TypeId>) -> bool;

    fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Output<'world>;
}

impl<T: 'static + Component> QueryFetch for &'_ T {
    type Output<'world> = &'world T;

    fn should_fetch(type_ids: &HashSet<TypeId>) -> bool {
        type_ids.contains(&TypeId::of::<T>())
    }

    fn fetch<'world>(_world: &'world World, entity: &Entity<'world>) -> Self::Output<'world> {
        entity.get_component::<T>().unwrap()
    }
}

impl QueryFetch for EntityId {
    type Output<'world> = EntityId;

    fn should_fetch(_type_ids: &HashSet<TypeId>) -> bool {
        true
    }

    fn fetch<'world>(_world: &'world World, entity: &Entity<'world>) -> Self::Output<'world> {
        entity.id()
    }
}

macro_rules! impl_query_fetch {
    ($($t:ident),*) => {
        impl<$($t: QueryFetch),*> QueryFetch for ($($t,)*) {
            type Output<'world> = ($($t::Output<'world>,)*);

            fn should_fetch(type_ids: &HashSet<TypeId>) -> bool {
                $($t::should_fetch(type_ids) &&)* true
            }

            fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Output<'world> {
                ($($t::fetch(world, entity),)*)
            }
        }
    };
  }

all_tuples!(impl_query_fetch, 1..16);

pub struct ComponentMut<'world, T: Component + Clone> {
    world: &'world World,
    id: EntityId,
    value: ManuallyDrop<Cow<'world, T>>,
}

impl<'world, T: Component + Clone> ComponentMut<'world, T> {
    pub fn new(world: &'world World, id: EntityId, value: &'world T) -> Self {
        Self {
            world,
            id,
            value: ManuallyDrop::new(Cow::Borrowed(value)),
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
        if let Cow::Owned(value) = unsafe { ManuallyDrop::take(&mut self.value) } {
            let change = ComponentChange::Added(Box::new(value));

            self.world
                .push_update(WorldUpdate::ModifyEntity(self.id, vec![change]))
        }
    }
}

impl<T: Component + Clone> QueryFetch for &'_ mut T {
    type Output<'world> = ComponentMut<'world, T>;

    fn should_fetch(type_ids: &HashSet<TypeId>) -> bool {
        type_ids.contains(&TypeId::of::<T>())
    }

    fn fetch<'world>(world: &'world World, entity: &Entity<'world>) -> Self::Output<'world> {
        ComponentMut::new(world, entity.id(), entity.get_component::<T>().unwrap())
    }
}
