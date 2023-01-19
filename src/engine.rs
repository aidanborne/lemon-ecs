use std::ops::{Deref, DerefMut};

use crate::{system::System, world::World};

/// A standalone collection of `System`s and a `World`.
#[derive(Default)]
pub struct Engine {
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl Engine {
    /// Pushes a `System` into the `Engine`s system list.
    /// This is typically a function taking a mutable reference to a `World`.
    #[inline]
    pub fn push_system(&mut self, system: impl System + 'static) -> &mut Self {
        self.systems.push(Box::new(system));
        self
    }

    /// Updates the interal `World` by running the added `System`s.
    pub fn run(&mut self) {
        self.systems.run(&mut self.world);
    }

    /// Used to run a single `System` on the `World` for initialization purposes.
    pub fn run_once(&mut self, mut system: impl System) -> &mut Self {
        system.run(&mut self.world);
        self
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
