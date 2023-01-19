use std::any::Any;

use crate::{
    collections::SparseSet,
    component::{Bundle, TypeBundle},
    entities::EntityId,
    world::World,
};

mod entity;

pub use entity::*;

/// Used to store changes to the `World` when they can't be applied immediately.
/// For example, iterating through a `Query` and modifying the `World` at the same time is not allowed.
#[derive(Default)]
pub struct WorldBuffer {
    existing_entities: SparseSet<ExistingEntityBuffer>,
    spawned_entities: Vec<SpawnedEntityBuffer>,
    despawned_entities: Vec<EntityId>,
    resources: Vec<Box<dyn Any>>,
}

impl WorldBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, bundle: impl Bundle) -> &mut SpawnedEntityBuffer {
        let mut buffer = SpawnedEntityBuffer::new();
        buffer.insert(bundle);

        self.spawned_entities.push(buffer);
        self.spawned_entities.last_mut().unwrap()
    }

    pub fn despawn(&mut self, id: EntityId) {
        self.despawned_entities.push(id);
        self.existing_entities.remove(*id);
    }

    pub fn insert(&mut self, id: EntityId, components: impl Bundle) -> &mut ExistingEntityBuffer {
        let buffer = self
            .existing_entities
            .get_or_insert_with(*id, || ExistingEntityBuffer::new(id));

        buffer.insert(components);
        buffer
    }

    pub fn remove<T: TypeBundle>(&mut self, id: EntityId) -> &mut ExistingEntityBuffer {
        let buffer = self
            .existing_entities
            .get_or_insert_with(*id, || ExistingEntityBuffer::new(id));

        buffer.remove::<T>();
        buffer
    }

    pub fn insert_resource(&mut self, resource: impl Any + 'static) {
        self.resources.push(Box::new(resource));
    }

    pub fn apply_world(self, world: &mut World) {
        for buffer in self.spawned_entities {
            world.spawn(buffer.into_components());
        }

        for id in self.despawned_entities {
            world.despawn(id);
        }

        for (id, buffer) in self.existing_entities.into_iter() {
            world.modify(id.into(), buffer.into_changes());
        }

        for resource in self.resources {
            world.insert_resource_boxed(resource);
        }
    }
}
