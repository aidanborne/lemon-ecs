use std::{
    any::{Any, TypeId},
    cell::RefCell,
};

use crate::component::{changes::ComponentChange, Component};

use super::{entities::EntityId, World};

pub enum WorldUpdate {
    SpawnEntity(EntityId, Vec<Box<dyn Component>>),
    DespawnEntity(EntityId),
    ModifyEntity(EntityId, Vec<ComponentChange>),
    InsertResource(Box<RefCell<dyn Any>>),
    RemoveResource(TypeId),
}

impl WorldUpdate {
    pub fn process(self, world: &mut World) {
        match self {
            WorldUpdate::SpawnEntity(id, components) => {
                world
                    .archetypes
                    .component_archetype(&components)
                    .insert(id, components);
            }
            WorldUpdate::DespawnEntity(id) => {
                world.despawn(id);
            }
            WorldUpdate::ModifyEntity(id, changes) => {
                world.modify_entity(id, changes.into_iter());
            }
            WorldUpdate::InsertResource(resource) => {
                let type_id = (*resource.borrow()).type_id();
                world.resources.insert(type_id, resource);
            }
            WorldUpdate::RemoveResource(type_id) => {
                world.resources.remove(&type_id);
            }
        };
    }
}
