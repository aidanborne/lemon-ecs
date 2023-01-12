use std::any::Any;

use crate::{
    collections::SparseSet,
    component::{Component, ComponentChange},
};

use super::{EntityId, World};

pub enum WorldUpdate {
    SpawnEntity(Vec<Box<dyn Component>>),
    DespawnEntity(EntityId),
    ModifyEntity(EntityId, Vec<ComponentChange>),
    InsertResource(Box<dyn Any>),
}

impl WorldUpdate {
    pub fn process(world: &mut World, updates: Vec<WorldUpdate>) {
        let mut entities: SparseSet<Vec<ComponentChange>> = SparseSet::new();

        for update in updates {
            match update {
                WorldUpdate::SpawnEntity(components) => {
                    world.spawn(components);
                }
                WorldUpdate::DespawnEntity(id) => {
                    world.despawn(id);
                    entities.remove(*id);
                }
                WorldUpdate::ModifyEntity(id, changes) => {
                    let vec = entities.get_or_insert_with(*id, Vec::new);
                    vec.extend(changes.into_iter())
                }
                WorldUpdate::InsertResource(resource) => {
                    world.resources.insert((*resource).type_id(), resource);
                }
            };
        }

        for (id, changes) in entities.into_iter() {
            world.modify_entity(id.into(), changes.into_iter());
        }
    }
}
