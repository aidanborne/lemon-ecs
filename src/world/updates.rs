use std::{
    any::{Any, TypeId},
    cell::RefCell,
};

use crate::{
    component::{bundle::ComponentBundle, changes::ComponentChange},
    storage::{archetypes::QueryResult, sparse_set::SparseSet},
};

use super::{entities::EntityId, World};

pub enum WorldUpdate {
    SpawnEntity(ComponentBundle),
    DespawnEntity(EntityId),
    ModifyEntity(EntityId, Vec<ComponentChange>),
    CacheQuery(TypeId, QueryResult),
    TrackChanges(TypeId),
    InsertResource(Box<RefCell<dyn Any>>),
    RemoveResource(TypeId),
}

impl WorldUpdate {
    pub fn process(self, world: &mut World) {
        match self {
            WorldUpdate::SpawnEntity(bundle) => {
                world.spawn(bundle);
            }
            WorldUpdate::DespawnEntity(id) => {
                world.despawn(id);
            }
            WorldUpdate::ModifyEntity(id, changes) => {
                world.modify_entity(*id, changes.into_iter());
            }
            WorldUpdate::CacheQuery(type_id, result) => {
                world.archetypes.cache_query(type_id, result);
            }
            WorldUpdate::TrackChanges(type_id) => {
                world.changes.insert(type_id, SparseSet::new());
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
