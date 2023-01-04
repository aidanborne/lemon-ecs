use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

use crate::world::World;

use super::params::SystemParameter;

#[repr(transparent)]
pub struct Resource<'world, T: 'static> {
    value: Ref<'world, T>,
}

impl<'world, T: 'static> Resource<'world, T> {
    pub fn new(value: Ref<'world, T>) -> Self {
        Self { value }
    }
}

impl<T> Deref for Resource<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

impl<T> SystemParameter for Option<Resource<'_, T>> {
    type Result<'world> = Option<Resource<'world, T>>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource::<T>()
    }
}

impl<T> SystemParameter for Resource<'_, T> {
    type Result<'world> = Resource<'world, T>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource::<T>().unwrap()
    }
}

#[repr(transparent)]
pub struct ResourceMut<'world, T: 'static> {
    value: RefMut<'world, T>,
}

impl<'world, T: 'static> ResourceMut<'world, T> {
    pub fn new(value: RefMut<'world, T>) -> Self {
        Self { value }
    }
}

impl<T> Deref for ResourceMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

impl<T> DerefMut for ResourceMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.value
    }
}

impl<T> SystemParameter for Option<ResourceMut<'_, T>> {
    type Result<'world> = Option<ResourceMut<'world, T>>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource_mut::<T>()
    }
}

impl<T> SystemParameter for ResourceMut<'_, T> {
    type Result<'world> = ResourceMut<'world, T>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.get_resource_mut::<T>().unwrap()
    }
}
