use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

use crate::world::World;

use super::params::SystemParameter;

#[repr(transparent)]
pub struct Res<'world, T: 'static> {
    value: Ref<'world, T>,
}

impl<'world, T: 'static> Res<'world, T> {
    pub fn new(value: Ref<'world, T>) -> Self {
        Self { value }
    }
}

impl<T> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

impl<T> SystemParameter for Option<Res<'_, T>> {
    type Result<'world> = Option<Res<'world, T>>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource::<T>()
    }
}

impl<T> SystemParameter for Res<'_, T> {
    type Result<'world> = Res<'world, T>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource::<T>().unwrap()
    }
}

#[repr(transparent)]
pub struct ResMut<'world, T: 'static> {
    value: RefMut<'world, T>,
}

impl<'world, T: 'static> ResMut<'world, T> {
    pub fn new(value: RefMut<'world, T>) -> Self {
        Self { value }
    }
}

impl<T> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

impl<T> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.value
    }
}

impl<T> SystemParameter for Option<ResMut<'_, T>> {
    type Result<'world> = Option<ResMut<'world, T>>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource_mut::<T>()
    }
}

impl<T> SystemParameter for ResMut<'_, T> {
    type Result<'world> = ResMut<'world, T>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource_mut::<T>().unwrap()
    }
}
