use std::{any::TypeId, collections::HashMap, iter::Enumerate, marker::PhantomData};

use crate::{
    collections::{sparse_set, ComponentVec, SparseSet},
    component::Component,
};

use super::EntityId;

pub(crate) struct Archetype {
    entities: SparseSet<PhantomData<bool>>,
    components: HashMap<TypeId, Box<dyn ComponentVec>>,
}

impl Archetype {
    pub fn from_components(components: &[Box<dyn Component>]) -> Self {
        let mut hash_map = HashMap::new();

        for component in components.iter() {
            hash_map.insert(component.as_any().type_id(), component.as_empty_vec());
        }

        Self {
            entities: SparseSet::new(),
            components: hash_map,
        }
    }

    /// Replaces the component of type `T` with the given component.
    pub fn replace_component(
        &mut self,
        id: EntityId,
        component: Box<dyn Component>,
    ) -> Box<dyn Component> {
        if let Some(idx) = self.entities.index_of(*id) {
            self.components
                .get_mut(&component.as_any().type_id())
                .and_then(|components| components.replace(idx, component))
                .unwrap_or_else(|| panic!("Entity did not have the given component."))
        } else {
            panic!("Entity does not exist in this archetype.")
        }
    }

    pub fn insert(&mut self, id: EntityId, components: Vec<Box<dyn Component>>) {
        if !self.entities.contains(*id) {
            self.entities.insert(*id, PhantomData);
        }

        let dense_idx = self.entities.index_of(*id).unwrap();

        for component in components {
            if let Some(storage) = self.components.get_mut(&component.as_any().type_id()) {
                storage.replace(dense_idx, component);
            }
        }
    }

    pub fn remove(&mut self, id: EntityId) -> Option<Vec<Box<dyn Component>>> {
        if let Some(idx) = self.entities.index_of(*id) {
            let mut components = Vec::new();

            for (_type_id, storage) in self.components.iter_mut() {
                components.push(storage.swap_remove(idx));
            }

            self.entities.remove(*id);
            Some(components)
        } else {
            None
        }
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains(*id)
    }

    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.components.contains_key(&type_id)
    }

    /// Returns a reference to the component of type `T` at the given dense index.
    pub fn get_dense_component<T: 'static + Component>(&self, loc: EntityLocation) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        if let Some(boxed_components) = self.components.get(&type_id) {
            if let Some(components) = boxed_components.downcast_ref::<Vec<T>>() {
                return components.as_slice().get(loc.idx());
            }
        }

        None
    }

    pub fn get_component<T: 'static + Component>(&self, id: EntityId) -> Option<&T> {
        if let Some(idx) = self.entities.index_of(*id) {
            return self.get_dense_component::<T>(EntityLocation::new(idx));
        }

        None
    }

    pub fn type_ids(&self) -> std::collections::HashSet<TypeId> {
        self.components.keys().cloned().collect()
    }

    pub fn iter(&self) -> ArchetypeIter<'_> {
        ArchetypeIter::new(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EntityLocation(usize);

impl EntityLocation {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

pub struct Entity<'archetype> {
    id: EntityId,
    archetype: &'archetype Archetype,
    location: EntityLocation,
}

impl<'archetype> Entity<'archetype> {
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn get_component<T: 'static + Component>(&self) -> Option<&'archetype T> {
        self.archetype.get_dense_component::<T>(self.location)
    }
}

pub(crate) struct ArchetypeIter<'archetype> {
    entities: Enumerate<sparse_set::Iter<'archetype, PhantomData<bool>>>,
    archetype: &'archetype Archetype,
}

impl<'archetype> ArchetypeIter<'archetype> {
    pub fn new(archetype: &'archetype Archetype) -> Self {
        Self {
            entities: archetype.entities.iter().enumerate(),
            archetype,
        }
    }
}

impl<'archetype> Iterator for ArchetypeIter<'archetype> {
    type Item = Entity<'archetype>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((idx, (id, _))) = self.entities.next() {
            Some(Entity {
                id: (*id).into(),
                archetype: self.archetype,
                location: EntityLocation::new(idx),
            })
        } else {
            None
        }
    }
}
