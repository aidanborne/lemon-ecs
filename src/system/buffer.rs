use std::any::TypeId;

use crate::{
    component::Bundleable,
    world::{World, WorldUpdate},
};

use super::params::SystemParameter;

pub struct SystemBuffer<'a> {
    world: &'a World,
}

impl<'a> SystemBuffer<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    pub fn spawn(&self, bundle: impl Bundleable) {
        self.world
            .push_update(WorldUpdate::SpawnEntity(bundle.bundle()));
    }

    pub fn despawn(&self, id: usize) {
        self.world.push_update(WorldUpdate::DespawnEntity(id));
    }

    pub fn insert(&self, id: usize, bundle: impl Bundleable) {
        self.world
            .push_update(WorldUpdate::InsertComponents(id, bundle.bundle()));
    }

    pub fn remove(&self, id: usize, types: Vec<TypeId>) {
        self.world
            .push_update(WorldUpdate::RemoveComponents(id, types));
    }
}

impl SystemParameter for SystemBuffer<'_> {
    type Result<'a> = SystemBuffer<'a>;

    fn resolve<'a>(world: &'a World) -> Self::Result<'a> {
        SystemBuffer::<'a>::new(world)
    }
}
