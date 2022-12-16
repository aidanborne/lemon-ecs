use std::{
    any::TypeId,
    collections::HashMap, fmt::Debug,
};

use crate::{
    component::{Component, ComponentBundle, ComponentStorage},
    query::Query,
};

use super::sparse_set::SparseSet;

pub struct Archetype {
    entities: SparseSet<usize>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            entities: SparseSet::new(),
            //edges: HashMap::new(),
            components: HashMap::new(),
        }
    }

    pub fn as_empty_with<T: 'static + Component>(&self) -> Self {
        let mut components: HashMap<TypeId, _> = HashMap::new();

        for (type_id, storage) in self.components.iter() {
            components.insert(*type_id, storage.as_empty_box());
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
                components.insert(*type_id, storage.as_empty_box());
            }
        }

        Self {
            entities: SparseSet::new(),
            components,
        }
    }

    pub fn replace<T: 'static + Component>(
        &mut self,
        id: usize,
        component: T,
    ) -> Result<(), ArchetypeError> {
        if !self.entities.contains(id) {
            return Err(ArchetypeError::MissingEntity(id));
        }

        let type_id = component.get_type_id();

        if let Some(component_storage) = self.components.get_mut(&type_id) {
            component_storage.insert(id, Box::new(component));
            Ok(())
        } else {
            Err(ArchetypeError::MissingStorage(type_id))
        }
    }

    pub fn insert(&mut self, id: usize, mut bundle: ComponentBundle) -> Result<(), ArchetypeError> {
        if !self.entities.contains(id) {
            self.entities.insert(id, id);
        }

        for (type_id, storage) in self.components.iter_mut() {
            if let Some(component) = bundle.remove(type_id) {
                storage.insert(id, component);
            } else {
                return Err(ArchetypeError::MissingBundle(id, *type_id));
            }
        }

        Ok(())
    }

    pub fn remove<'a>(&'a mut self, id: usize) -> Result<ComponentBundle, ArchetypeError> {
        if !self.entities.contains(id) {
            return Err(ArchetypeError::MissingEntity(id));
        }

        self.entities.remove(id);

        let mut bundle = ComponentBundle::new();

        for (type_id, storage) in self.components.iter_mut() {
            if let Some(component) = storage.remove(id) {
                bundle.insert(component);
            } else {
                return Err(ArchetypeError::MissingComponent(id, *type_id));
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

    pub fn get_component<T: 'static + Component>(&self, id: usize) -> Result<&T, ArchetypeError> {
        let type_id = TypeId::of::<T>();

        if let Some(component_storage) = self.components.get(&type_id) {
            if let Some(component) = component_storage.as_any().downcast_ref::<SparseSet<T>>() {
                return component
                    .get(id)
                    .ok_or(ArchetypeError::MissingComponent(id, type_id));
            }
        }

        Err(ArchetypeError::MissingStorage(type_id))
    }

    pub fn get_query(&self) -> Query {
        Query::new(self.components.keys().cloned().collect())
    }

    pub fn iter(&self) -> crate::sparse_set::Keys<usize> {
        self.entities.keys()
    }
}

#[derive(Clone, Copy)]
pub enum ArchetypeError {
    MissingBundle(usize, TypeId),
    MissingStorage(TypeId),
    MissingComponent(usize, TypeId),
    MissingEntity(usize),
}

impl Debug for ArchetypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBundle(id, type_id) => write!(f, "Entity #{} is missing component with {:?} in bundle", id, type_id),
            Self::MissingStorage(type_id) => write!(f, "Archetype is missing storage for {:?}", type_id),
            Self::MissingComponent(id, type_id) => write!(f, "Entity #{} is missing component with {:?} in archetype", id, type_id),
            Self::MissingEntity(id) => write!(f, "Entity #{} is missing in archetype", id),
        }
    }
}