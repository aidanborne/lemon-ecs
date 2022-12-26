use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use crate::{
    component::{Bundleable, Component, ComponentBundle},
    query::{Query, Queryable},
    storage::archetypes::{self, ArchetypeArena, QueryResult},
    system::resource::{Resource, ResourceMut},
};

pub enum WorldUpdate {
    SpawnEntity(ComponentBundle),
    DespawnEntity(usize),
    InsertComponents(usize, ComponentBundle),
    RemoveComponents(usize, Vec<TypeId>),
    CacheQuery(TypeId, QueryResult),
    InsertResource(Box<RefCell<dyn Any>>),
    RemoveResource(TypeId),
}

pub struct World {
    available_ids: Vec<usize>,
    next_id: usize,
    archetypes: ArchetypeArena,
    updates: RefCell<Vec<WorldUpdate>>,
    resources: HashMap<TypeId, Box<RefCell<dyn Any>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            available_ids: Vec::new(),
            next_id: 0,
            archetypes: Default::default(),
            updates: Default::default(),
            resources: Default::default(),
        }
    }

    pub fn spawn(&mut self, bundle: impl Bundleable) -> usize {
        let id = match self.available_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 1;
                id
            }
        };

        let bundle = bundle.bundle();

        let idx = self.archetypes.get_bundle_archetype(&bundle);

        self.archetypes[idx].insert(id, bundle);
        id
    }

    pub fn spawn_empty(&mut self) -> usize {
        self.spawn(ComponentBundle::new())
    }

    pub fn despawn(&mut self, id: usize) -> Option<ComponentBundle> {
        if let Some(idx) = self.archetypes.get_entity_archetype(id) {
            let bundle = self.archetypes[idx].remove(id);
            self.available_ids.push(id);
            bundle
        } else {
            None
        }
    }

    pub fn has_component<T: 'static + Component>(&self, id: usize) -> bool {
        self.archetypes
            .get_entity_archetype(id)
            .map(|idx| {
                self.archetypes[idx].contains(id)
                    && self.archetypes[idx].has_component(TypeId::of::<T>())
            })
            .unwrap_or(false)
    }

    pub fn add_component<T: 'static + Component>(&mut self, id: usize, component: T) {
        if let Some(curr_idx) = self.archetypes.get_entity_archetype(id) {
            if self.archetypes[curr_idx].has_component(TypeId::of::<T>()) {
                self.archetypes[curr_idx].replace_component(id, component);
            } else {
                let mut bundle = self.archetypes[curr_idx].remove(id).unwrap();
                bundle.push(Box::new(component));

                let new_idx = self.archetypes.get_bundle_archetype(&bundle);
                self.archetypes[new_idx].insert(id, bundle);
            }
        }
    }

    pub fn remove_component<T: 'static + Component>(&mut self, id: usize) -> Option<T> {
        if let Some(idx) = self.archetypes.get_entity_archetype(id) {
            if self.archetypes[idx].has_component(TypeId::of::<T>()) {
                let mut bundle = self.archetypes[idx].remove(id).unwrap();

                let comp_idx = bundle
                    .iter()
                    .position(|component| component.component_id() == TypeId::of::<T>())
                    .unwrap();

                let component = bundle.swap_remove(comp_idx);
                self.archetypes[idx].insert(id, bundle);

                return component.downcast::<T>().ok().map(|component| *component);
            }
        }

        None
    }

    pub fn get_component<T: 'static + Component>(&self, id: usize) -> Option<&T> {
        self.archetypes
            .get_entity_archetype(id)
            .and_then(|idx| self.archetypes[idx].get_component::<T>(id))
    }

    pub fn update_entity<F: FnOnce(&mut ComponentBundle)>(&mut self, id: usize, func: F) {
        if let Some(idx) = self.archetypes.get_entity_archetype(id) {
            let mut bundle = self.archetypes[idx].remove(id).unwrap();
            func(&mut bundle);

            let new_idx = self.archetypes.get_bundle_archetype(&bundle);
            self.archetypes[new_idx].insert(id, bundle);
        }
    }

    pub fn query<'a, T: Queryable<'a>>(&'a self) -> Query<'a, T::Fetch, T::Filter> {
        let type_id = TypeId::of::<(T::Fetch, T::Filter)>();

        match self.archetypes.query_cached(type_id) {
            Some(result) => Query::new(archetypes::Iter::new(
                &self.archetypes,
                result.archetypes.clone(),
            )),
            None => {
                let result = self.archetypes.query_uncached(T::get_pattern());
                let ids = result.archetypes.clone();

                self.updates
                    .borrow_mut()
                    .push(WorldUpdate::CacheQuery(type_id, result));

                Query::new(archetypes::Iter::new(&self.archetypes, ids))
            }
        }
    }

    pub fn get_resource<T: 'static>(&self) -> Option<Resource<T>> {
        let cell = &**self.resources.get(&TypeId::of::<T>())?;
        let ref_value = Ref::map(cell.borrow(), |r| r.downcast_ref::<T>().unwrap());
        Some(Resource::new(ref_value))
    }

    pub fn get_resource_mut<T: 'static>(&self) -> Option<ResourceMut<T>> {
        let cell = &**self.resources.get(&TypeId::of::<T>())?;
        let ref_value = RefMut::map(cell.borrow_mut(), |r| r.downcast_mut::<T>().unwrap());
        Some(ResourceMut::new(ref_value))
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources
            .insert(TypeId::of::<T>(), Box::new(RefCell::new(resource)));
    }

    pub fn remove_resource<T: 'static>(&mut self) {
        self.resources.remove(&TypeId::of::<T>());
    }

    pub fn process_updates(&mut self) {
        for update in self.updates.replace(Vec::new()).into_iter() {
            match update {
                WorldUpdate::SpawnEntity(bundle) => {
                    self.spawn(bundle);
                }
                WorldUpdate::DespawnEntity(id) => {
                    self.despawn(id);
                }
                WorldUpdate::InsertComponents(id, bundle) => {
                    if let Some(idx) = self.archetypes.get_entity_archetype(id) {
                        let mut curr_bundle = self.archetypes[idx].remove(id).unwrap();
                        curr_bundle.extend(bundle);
                        self.archetypes[idx].insert(id, curr_bundle);
                    }
                }
                WorldUpdate::RemoveComponents(id, types) => {
                    if let Some(idx) = self.archetypes.get_entity_archetype(id) {
                        let mut curr_bundle = self.archetypes[idx].remove(id).unwrap();
                        curr_bundle.retain(|component| !types.contains(&component.component_id()));
                        self.archetypes[idx].insert(id, curr_bundle);
                    }
                }
                WorldUpdate::CacheQuery(type_id, result) => {
                    self.archetypes.cache_query(type_id, result);
                }
                WorldUpdate::InsertResource(resource) => {
                    let type_id = (*resource.borrow()).type_id();
                    self.resources.insert(type_id, resource);
                }
                WorldUpdate::RemoveResource(type_id) => {
                    self.resources.remove(&type_id);
                }
            };
        }
    }

    pub fn push_update(&self, update: WorldUpdate) {
        self.updates.borrow_mut().push(update);
    }
}
