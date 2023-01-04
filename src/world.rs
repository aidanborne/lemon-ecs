use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
};

use crate::{
    component::{
        bundle::{Bundleable, ComponentBundle},
        changes::{ChangeRecord, ComponentChange},
        Component,
    },
    query::{fetch::QueryFetch, filter::QueryFilter, Query, QueryChanged},
    storage::{
        archetypes::{ArchetypeArena, QueryResult},
        sparse_set::SparseSet,
    },
    system::resource::{Resource, ResourceMut},
};

pub enum WorldUpdate {
    SpawnEntity(ComponentBundle),
    DespawnEntity(usize),
    ModifyEntity(usize, Vec<ComponentChange>),
    CacheQuery(TypeId, QueryResult),
    TrackChanges(TypeId),
    InsertResource(Box<RefCell<dyn Any>>),
    RemoveResource(TypeId),
}

pub struct World {
    available_ids: Vec<usize>,
    next_id: usize,
    archetypes: ArchetypeArena,
    updates: RefCell<Vec<WorldUpdate>>,
    resources: HashMap<TypeId, Box<RefCell<dyn Any>>>,
    changes: HashMap<TypeId, SparseSet<ChangeRecord>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            available_ids: Vec::new(),
            next_id: 0,
            archetypes: Default::default(),
            updates: Default::default(),
            resources: Default::default(),
            changes: Default::default(),
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

    fn get_component_record(&mut self, id: usize, type_id: TypeId) -> &mut ChangeRecord {
        let sparse_set = self.changes.entry(type_id).or_insert_with(SparseSet::new);

        if !sparse_set.contains(id) {
            sparse_set.insert(id, ChangeRecord::default());
        }

        sparse_set.get_mut(id).unwrap()
    }

    fn modify_bundle(
        &mut self,
        id: usize,
        bundle: ComponentBundle,
        changes: impl Iterator<Item = ComponentChange>,
    ) {
        let mut components: HashMap<TypeId, Box<dyn Component>> = bundle
            .into_iter()
            .map(|component| (component.as_any().type_id(), component))
            .collect();

        for change in changes {
            match change {
                ComponentChange::Added(component) => {
                    let type_id = component.as_any().type_id();
                    let removed = components.insert(type_id, component);
                    self.get_component_record(id, type_id).added(removed);
                }
                ComponentChange::Removed(type_id) => {
                    let removed = components.remove(&type_id);

                    if let Some(component) = removed {
                        self.get_component_record(id, type_id).removed(component);
                    }
                }
            }
        }

        let bundle: ComponentBundle = components
            .into_iter()
            .map(|(_, component)| component)
            .collect();

        let archetype_id = self.archetypes.get_bundle_archetype(&bundle);
        self.archetypes[archetype_id].insert(id, bundle);
    }

    fn modify_entity(&mut self, id: usize, mut changes: impl Iterator<Item = ComponentChange>) {
        let curr_archetype_id = self.archetypes.get_entity_archetype(id);

        if curr_archetype_id.is_none() {
            return;
        }

        let curr_archetype_id = curr_archetype_id.unwrap();
        let archetype = self.archetypes[curr_archetype_id].get_archetype();
        let hash_set: HashSet<&TypeId> = archetype.iter().collect();

        let mut breaking_change = None;

        while let Some(change) = changes.next() {
            match change {
                ComponentChange::Added(component) => {
                    let type_id = component.as_any().type_id();

                    if hash_set.contains(&type_id) {
                        let removed =
                            self.archetypes[curr_archetype_id].replace_component(id, component);
                        self.get_component_record(id, type_id).added(removed);
                    } else {
                        breaking_change = Some(ComponentChange::Added(component));
                        break;
                    }
                }
                ComponentChange::Removed(type_id) => {
                    if hash_set.contains(&type_id) {
                        breaking_change = Some(change);
                        break;
                    }
                }
            }
        }

        if let Some(breaking_change) = breaking_change {
            let bundle = self.archetypes[curr_archetype_id].remove(id).unwrap();
            self.modify_bundle(id, bundle, std::iter::once(breaking_change).chain(changes));
        }
    }

    pub fn insert(&mut self, id: usize, components: impl Bundleable) {
        self.modify_entity(
            id,
            components.bundle().into_iter().map(ComponentChange::Added),
        );
    }

    pub fn remove(&mut self, id: usize, types: &[TypeId]) {
        self.modify_entity(id, types.iter().cloned().map(ComponentChange::Removed));
    }

    pub fn get_component<T: 'static + Component>(&self, id: usize) -> Option<&T> {
        self.archetypes
            .get_entity_archetype(id)
            .and_then(|idx| self.archetypes[idx].get_component::<T>(id))
    }

    pub fn query<Fetch: 'static + QueryFetch, Filter: 'static + QueryFilter>(
        &self,
    ) -> Query<Fetch, Filter> {
        let type_id = TypeId::of::<(Fetch, Filter)>();

        match self.archetypes.query_cached(type_id) {
            Some(result) => Query::new(&self.archetypes, result.archetypes.clone()),
            None => {
                let result = self
                    .archetypes
                    .query_uncached(Query::<Fetch, Filter>::get_pattern());

                let ids = result.archetypes.clone();

                self.updates
                    .borrow_mut()
                    .push(WorldUpdate::CacheQuery(type_id, result));

                Query::new(&self.archetypes, ids)
            }
        }
    }

    pub fn track_changes(&mut self, type_id: TypeId) {
        self.changes.entry(type_id).or_insert_with(SparseSet::new);
    }

    // TODO: Don't return an Option
    pub fn query_changed<T: 'static + Component>(&self) -> Option<QueryChanged<T>> {
        let type_id = TypeId::of::<T>();

        if let Some(changes) = self.changes.get(&type_id) {
            Some(QueryChanged::new(&self, changes.iter()))
        } else {
            self.push_update(WorldUpdate::TrackChanges(type_id));
            None
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
                WorldUpdate::ModifyEntity(id, changes) => {
                    self.modify_entity(id, changes.into_iter());
                }
                WorldUpdate::CacheQuery(type_id, result) => {
                    self.archetypes.cache_query(type_id, result);
                }
                WorldUpdate::TrackChanges(type_id) => {
                    self.changes.insert(type_id, SparseSet::new());
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

        for (_, changes) in self.changes.iter_mut() {
            changes.clear();
        }
    }

    pub fn push_update(&self, update: WorldUpdate) {
        self.updates.borrow_mut().push(update);
    }
}
