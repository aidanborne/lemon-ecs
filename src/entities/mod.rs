use std::ops::Deref;

use crate::component::Component;

mod archetype;
mod archetypes;
mod iter;

pub use archetype::*;
pub(crate) use archetypes::*;
pub use iter::*;

pub(crate) struct IdGenerator {
    available_ids: Vec<EntityId>,
    next_id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            available_ids: Vec::new(),
            next_id: 0,
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        match self.available_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 1;
                id.into()
            }
        }
    }

    pub fn despawn(&mut self, id: EntityId) {
        self.available_ids.push(id);
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct EntityId(usize);

impl EntityId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl From<usize> for EntityId {
    #[inline]
    fn from(id: usize) -> Self {
        Self::new(id)
    }
}

impl From<EntityId> for usize {
    #[inline]
    fn from(id: EntityId) -> Self {
        id.0
    }
}

impl Deref for EntityId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Entity<'archetype> {
    id: EntityId,
    archetype: &'archetype Archetype,
    idx: usize,
}

impl<'archetype> Entity<'archetype> {
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn get_component<T: 'static + Component>(&self) -> Option<&'archetype T> {
        self.archetype.get_component_dense::<T>(self.idx)
    }
}
