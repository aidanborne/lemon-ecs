use std::{any::TypeId, mem::ManuallyDrop};

use crate::{
    component::{Bundle, ComponentChange, TypeBundle},
    world::{EntityId, World, WorldUpdate},
};

pub struct WorldBuffer<'world> {
    world: &'world World,
}

impl<'world> WorldBuffer<'world> {
    pub fn new(world: &'world World) -> Self {
        Self { world }
    }

    pub fn spawn(&self, components: impl Bundle) -> EntityBuffer<'world> {
        let mut buffer = EntityBuffer::spawn(self.world);
        buffer.insert(components);
        buffer
    }

    pub fn despawn(&self, id: EntityId) {
        self.world.push_update(WorldUpdate::DespawnEntity(id));
    }

    pub fn insert(&self, id: EntityId, components: impl Bundle) -> EntityBuffer<'world> {
        let mut buffer = EntityBuffer::new(self.world, id);
        buffer.insert(components);
        buffer
    }

    pub fn remove<T: TypeBundle>(&self, id: EntityId) -> EntityBuffer<'world> {
        let mut buffer = EntityBuffer::new(self.world, id);
        buffer.remove::<T>();
        buffer
    }

    pub fn insert_resource<T: 'static>(&self, resource: T) {
        self.world
            .push_update(WorldUpdate::InsertResource(Box::new(resource)));
    }

    pub fn remove_resource<T: 'static>(&self) {
        self.world
            .push_update(WorldUpdate::RemoveResource(TypeId::of::<T>()));
    }
}

pub struct EntityBuffer<'world> {
    world: &'world World,
    id: Option<EntityId>,
    changes: ManuallyDrop<Vec<ComponentChange>>,
}

impl<'world> EntityBuffer<'world> {
    pub fn spawn(world: &'world World) -> Self {
        Self {
            world,
            id: None,
            changes: ManuallyDrop::new(Vec::new()),
        }
    }

    pub fn new(world: &'world World, id: EntityId) -> Self {
        Self {
            world,
            id: Some(id),
            changes: ManuallyDrop::new(Vec::new()),
        }
    }

    pub fn insert(&mut self, components: impl Bundle) -> &Self {
        for component in components.components() {
            self.changes.push(ComponentChange::Added(component));
        }

        self
    }

    pub fn remove<T: TypeBundle>(&mut self) -> &Self {
        for type_id in T::type_ids() {
            self.changes.push(ComponentChange::Removed(type_id));
        }

        self
    }
}

impl<'world> Drop for EntityBuffer<'world> {
    fn drop(&mut self) {
        let changes = unsafe { ManuallyDrop::take(&mut self.changes).into_iter() };

        let update = match self.id {
            Some(id) => WorldUpdate::ModifyEntity(id, changes.collect()),
            None => {
                let components = changes
                    .filter_map(|change| {
                        if let ComponentChange::Added(component) = change {
                            Some(component)
                        } else {
                            None
                        }
                    })
                    .collect();

                WorldUpdate::SpawnEntity(components)
            }
        };

        self.world.push_update(update);
    }
}
