use std::ops::{Deref, DerefMut};

use crate::{
    system::{BoxedSystem, IntoSystem},
    world::World,
};

pub struct Engine {
    world: World,
    systems: Vec<BoxedSystem>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            systems: Vec::new(),
        }
    }

    /// Add a system to the engine.
    pub fn add_system<T>(&mut self, system: impl IntoSystem<T>) {
        self.systems.push(system.into_system());
    }

    /// Update the engine.
    pub fn update(&mut self) {
        for system in self.systems.iter() {
            system.update(&self.world);
        }
    }
}

impl Deref for Engine {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for Engine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}
