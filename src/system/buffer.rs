use std::{any::TypeId, cell::RefCell};

use crate::{
    component::{bundle::Bundleable, changes::ComponentChange},
    world::{entities::EntityId, updates::WorldUpdate, World},
};

use super::params::SystemParameter;

pub struct SystemBuffer<'world> {
    world: &'world World,
}

impl<'world> SystemBuffer<'world> {
    pub fn new(world: &'world World) -> Self {
        Self { world }
    }

    pub fn spawn(&self, bundle: impl Bundleable) {
        self.world
            .push_update(WorldUpdate::SpawnEntity(bundle.bundle()));
    }

    pub fn despawn(&self, id: EntityId) {
        self.world.push_update(WorldUpdate::DespawnEntity(id));
    }

    pub fn insert(&self, id: EntityId, components: impl Bundleable) {
        let changes = components
            .bundle()
            .into_iter()
            .map(|component| ComponentChange::Added(component))
            .collect();

        self.world
            .push_update(WorldUpdate::ModifyEntity(id, changes));
    }

    pub fn remove(&self, id: EntityId, types: &[TypeId]) {
        let types = types
            .iter()
            .map(|type_id| ComponentChange::Removed(*type_id))
            .collect();

        self.world.push_update(WorldUpdate::ModifyEntity(id, types));
    }

    pub fn insert_resource<T: 'static>(&self, resource: T) {
        self.world
            .push_update(WorldUpdate::InsertResource(Box::new(RefCell::new(
                resource,
            ))));
    }

    pub fn remove_resource<T: 'static>(&self) {
        self.world
            .push_update(WorldUpdate::RemoveResource(TypeId::of::<T>()));
    }
}

impl SystemParameter for SystemBuffer<'_> {
    type Result<'world> = SystemBuffer<'world>;

    fn resolve(world: &World) -> Self::Result<'_> {
        SystemBuffer::new(world)
    }
}
