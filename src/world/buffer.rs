use std::{any::TypeId, cell::RefCell};

use crate::{
    component::{bundle::Bundleable, changes::ComponentChange},
    world::{entities::EntityId, updates::WorldUpdate, World},
};

pub struct WorldBuffer<'world> {
    world: &'world World,
}

impl<'world> WorldBuffer<'world> {
    pub fn new(world: &'world World) -> Self {
        Self { world }
    }

    pub fn spawn(&self, components: impl Bundleable) -> EntityBuffer<'world> {
        let id = self.world.entities.borrow_mut().spawn().into();

        self.world
            .push_update(WorldUpdate::SpawnEntity(id, components.bundle()));

        EntityBuffer::new(self.world, id)
    }

    pub fn despawn(&self, id: EntityId) {
        self.world.push_update(WorldUpdate::DespawnEntity(id));
    }

    pub fn insert(&self, id: EntityId, components: impl Bundleable) -> EntityBuffer<'world> {
        let buffer = EntityBuffer::new(self.world, id);
        buffer.insert(components);
        buffer
    }

    pub fn remove<Iter>(&self, id: EntityId, types: Iter) -> EntityBuffer<'world>
    where
        Iter: IntoIterator<Item = TypeId>,
    {
        let buffer = EntityBuffer::new(self.world, id);
        buffer.remove(types);
        buffer
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

pub struct EntityBuffer<'world> {
    world: &'world World,
    id: EntityId,
}

impl<'world> EntityBuffer<'world> {
    pub fn new(world: &'world World, id: EntityId) -> Self {
        Self { world, id }
    }

    pub fn insert(&self, components: impl Bundleable) -> &Self {
        let changes = components
            .bundle()
            .into_iter()
            .map(|component| ComponentChange::Added(component))
            .collect();

        self.world
            .push_update(WorldUpdate::ModifyEntity(self.id, changes));

        self
    }

    pub fn remove<Iter>(&self, types: Iter) -> &Self
    where
        Iter: IntoIterator<Item = TypeId>,
    {
        let types = types.into_iter().map(ComponentChange::Removed).collect();

        self.world
            .push_update(WorldUpdate::ModifyEntity(self.id, types));

        self
    }
}
