use std::{
    borrow::Cow,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use crate::world::{World, WorldUpdate};

use super::params::SystemParameter;

pub trait ResParameter {
    type Resource;
    type Output<'world>: Deref<Target = Self::Resource>;

    fn from_world(world: &World) -> Option<Self::Output<'_>>;
}

impl<T: ResParameter> SystemParameter for Option<T> {
    type Result<'world> = Option<T::Output<'world>>;

    fn resolve(world: &World) -> Self::Result<'_> {
        T::from_world(world)
    }
}

impl<T: ResParameter> SystemParameter for T {
    type Result<'world> = T::Output<'world>;

    fn resolve(world: &World) -> Self::Result<'_> {
        T::from_world(world).unwrap_or_else(|| {
            panic!(
                "Resource '{}' not found in world",
                std::any::type_name::<T::Resource>()
            )
        })
    }
}

#[repr(transparent)]
pub struct Res<'world, T: 'static>(&'world T);

impl<'world, T: 'static> Res<'world, T> {
    pub fn new(value: &'world T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> ResParameter for Res<'_, T> {
    type Resource = T;
    type Output<'world> = Res<'world, T>;

    fn from_world(world: &World) -> Option<Self::Output<'_>> {
        world.get_resource::<T>().map(Res::new)
    }
}

pub struct ResMut<'world, T: Clone + 'static> {
    world: &'world World,
    value: ManuallyDrop<Cow<'world, T>>,
}

impl<'world, T: Clone + 'static> ResMut<'world, T> {
    pub fn new(world: &'world World, value: &'world T) -> Self {
        Self {
            world,
            value: ManuallyDrop::new(Cow::Borrowed(value)),
        }
    }
}

impl<T: Clone> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Clone> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.to_mut()
    }
}

impl<T: Clone> Drop for ResMut<'_, T> {
    fn drop(&mut self) {
        if let Cow::Owned(owned) = unsafe { ManuallyDrop::take(&mut self.value) } {
            self.world
                .push_update(WorldUpdate::InsertResource(Box::new(owned)));
        }
    }
}

impl<T: Clone> ResParameter for ResMut<'_, T> {
    type Resource = T;
    type Output<'world> = ResMut<'world, T>;

    fn from_world(world: &World) -> Option<Self::Output<'_>> {
        world
            .get_resource::<T>()
            .map(|value| ResMut::new(world, value))
    }
}
