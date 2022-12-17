use std::{any::TypeId, collections::HashMap, fmt::Debug};

use crate::{component::Component, query::Archetype};

use super::{bundle::ComponentBundle, components::ComponentStorage, sparse_set::SparseSet};

pub struct EntityStorage {
    entities: SparseSet<usize>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl EntityStorage {
    pub fn new() -> Self {
        Self {
            entities: SparseSet::new(),
            components: HashMap::new(),
        }
    }

    pub fn as_empty_with<T: 'static + Component>(&self) -> Self {
        let mut components: HashMap<TypeId, _> = HashMap::new();

        for (type_id, storage) in self.components.iter() {
            components.insert(*type_id, storage.as_empty_boxed());
        }

        components.insert(TypeId::of::<T>(), Box::new(SparseSet::<T>::new()));

        Self {
            entities: SparseSet::new(),
            components,
        }
    }

    pub fn as_empty_without<T: 'static + Component>(&self) -> Self {
        let mut components: HashMap<TypeId, _> = HashMap::new();

        for (type_id, storage) in self.components.iter() {
            if *type_id != TypeId::of::<T>() {
                components.insert(*type_id, storage.as_empty_boxed());
            }
        }

        Self {
            entities: SparseSet::new(),
            components,
        }
    }

    pub fn replace_component<T: 'static + Component>(
        &mut self,
        id: usize,
        component: T,
    ) -> Result<(), EntityErr> {
        if !self.entities.contains(id) {
            return Err(EntityErr::MissingEntityInArchetype(id));
        }

        let type_id = TypeId::of::<T>();

        if let Some(component_storage) = self.components.get_mut(&type_id) {
            component_storage.insert(id, Box::new(component));
            Ok(())
        } else {
            Err(EntityErr::MissingStorageInArchetype(type_id))
        }
    }

    pub fn insert(&mut self, id: usize, mut bundle: ComponentBundle) -> Result<(), EntityErr> {
        if !self.entities.contains(id) {
            self.entities.insert(id, id);
        }

        for (type_id, storage) in self.components.iter_mut() {
            if let Some(component) = bundle.remove(type_id) {
                storage.insert(id, component);
            } else {
                return Err(EntityErr::MissingComponentInBundle(id, *type_id));
            }
        }

        Ok(())
    }

    pub fn remove<'a>(&'a mut self, id: usize) -> Result<ComponentBundle, EntityErr> {
        if !self.entities.contains(id) {
            return Err(EntityErr::MissingEntityInArchetype(id));
        }

        self.entities.remove(id);

        let mut bundle = ComponentBundle::new();

        for (type_id, storage) in self.components.iter_mut() {
            if let Some(component) = storage.remove(id) {
                bundle.insert(component);
            } else {
                return Err(EntityErr::MissingComponentInArchetype(id, *type_id));
            }
        }

        Ok(bundle)
    }

    pub fn contains(&self, id: usize) -> bool {
        self.entities.contains(id)
    }

    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.components.contains_key(&type_id)
    }

    pub fn get_component<T: 'static + Component>(&self, id: usize) -> Result<&T, EntityErr> {
        let type_id = TypeId::of::<T>();

        if let Some(component_storage) = self.components.get(&type_id) {
            if let Some(component) = component_storage.as_any().downcast_ref::<SparseSet<T>>() {
                return component
                    .get(id)
                    .ok_or(EntityErr::MissingComponentInArchetype(id, type_id));
            }
        }

        Err(EntityErr::MissingStorageInArchetype(type_id))
    }

    pub fn get_archetype(&self) -> Archetype {
        self.components.keys().cloned().collect()
    }

    pub fn iter(&self) -> super::sparse_set::Keys<usize> {
        self.entities.keys()
    }
}

#[derive(Clone, Copy)]
pub enum EntityErr {
    MissingComponentInBundle(usize, TypeId),
    MissingStorageInArchetype(TypeId),
    MissingComponentInArchetype(usize, TypeId),
    MissingEntityInArchetype(usize),
}

impl Debug for EntityErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingComponentInBundle(id, type_id) => {
                write!(f, "Bundle is missing {:?} for entity #{}", type_id, id)
            }
            Self::MissingStorageInArchetype(type_id) => {
                write!(f, "Archetype is missing storage for {:?}", type_id)
            }
            Self::MissingComponentInArchetype(id, type_id) => {
                write!(f, "Archetype is missing {:?} for entity #{}", type_id, id)
            }
            Self::MissingEntityInArchetype(id) => write!(f, "Entity #{} is not in archetype", id),
        }
    }
}
