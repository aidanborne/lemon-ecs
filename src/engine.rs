use std::ops::{Deref, DerefMut};

use crate::{system::System, world::World};

#[derive(Default)]
pub struct Engine {
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl Engine {
    /// Adds a system to the engine.
    /// This is typically a function that takes a mutable reference to the world.
    #[inline]
    pub fn add_system(&mut self, system: impl System + 'static) {
        self.systems.push(Box::new(system));
    }

    /// Updates the engine by running all systems in order.
    pub fn update(&mut self) {
        for system in self.systems.iter_mut() {
            system.update(&mut self.world);
        }

        self.world.process_updates();
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
