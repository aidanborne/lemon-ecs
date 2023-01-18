use std::ops::{Deref, DerefMut};

use crate::{
    system::{IntoSystem, System},
    world::World,
};

pub struct Engine {
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            world: World::default(),
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

        self.world.process_updates();
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
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
