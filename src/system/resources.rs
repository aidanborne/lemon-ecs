use std::ops::Deref;

use crate::{traits::AsAny, world::World};

use super::params::SystemParameter;

/// Represents a non-removable singleton within a world.
pub trait Resource: AsAny {
    /// Called each time updates to the world are applied.
    fn update(&mut self) {}
}

#[repr(transparent)]
pub struct Res<'world, T: 'static>(&'world T);

impl<'world, T: 'static + Resource> Res<'world, T> {
    pub fn new(value: &'world T) -> Self {
        Self(value)
    }
}

impl<T: Resource> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: Resource> SystemParameter for Option<Res<'_, T>> {
    type Output<'world> = Option<Res<'world, T>>;

    fn resolve(world: &World) -> Self::Output<'_> {
        world.get_resource::<T>().map(Res::new)
    }
}

impl<T: Resource> SystemParameter for Res<'_, T> {
    type Output<'world> = Res<'world, T>;

    fn resolve(world: &World) -> Self::Output<'_> {
        Option::<Res<'_, T>>::resolve(world).unwrap_or_else(|| {
            panic!(
                "Resource '{}' not found in world",
                std::any::type_name::<T>()
            )
        })
    }
}
