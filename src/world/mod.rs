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
    storage::{archetypes::Archetypes, sparse_set::SparseSet},
    system::resource::{Resource, ResourceMut},
};

use self::{entities::EntityId, updates::WorldUpdate};

pub mod entities;
pub mod updates;

#[derive(Default)]
pub struct World {
    entities: entities::Entities,
    archetypes: Archetypes,
    updates: RefCell<Vec<WorldUpdate>>,
    resources: HashMap<TypeId, Box<RefCell<dyn Any>>>,
    changes: HashMap<TypeId, SparseSet<ChangeRecord>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, components: impl Bundleable) -> EntityId {
        let id = self.entities.spawn().into();

        let bundle = components.bundle();

        let archetype = self.archetypes.get_bundle_archetype(&bundle);
        self.archetypes[archetype].insert(id, bundle);

        id
    }

    pub fn despawn(&mut self, id: EntityId) -> Option<ComponentBundle> {
        if let Some(idx) = self.archetypes.get_entity_archetype(id) {
            let bundle = self.archetypes[idx].remove(id);
            self.entities.despawn(*id);
            bundle
        } else {
            None
        }
    }

    pub fn has_component<T: 'static + Component>(&self, id: EntityId) -> bool {
        self.archetypes
            .get_entity_archetype(id)
            .map(|idx| {
                self.archetypes[idx].contains(id)
                    && self.archetypes[idx].has_component(TypeId::of::<T>())
            })
            .unwrap_or(false)
    }

    fn get_component_record(&mut self, id: EntityId, type_id: TypeId) -> Option<&mut ChangeRecord> {
        let sparse_set = self.changes.get_mut(&type_id)?;
        let id = *id;

        if !sparse_set.contains(id) {
            sparse_set.insert(id, ChangeRecord::new());
        }

        sparse_set.get_mut(id)
    }

    fn modify_bundle<Iter>(&mut self, id: EntityId, bundle: ComponentBundle, changes: Iter)
    where
        Iter: Iterator<Item = ComponentChange>,
    {
        let mut components: HashMap<TypeId, Box<dyn Component>> = bundle
            .into_iter()
            .map(|component| ((*component).as_any().type_id(), component))
            .collect();

        for change in changes {
            match change {
                ComponentChange::Added(component) => {
                    let type_id = (*component).as_any().type_id();
                    let removed = components.insert(type_id, component);

                    if let Some(record) = self.get_component_record(id, type_id) {
                        record.added(removed);
                    }
                }
                ComponentChange::Removed(type_id) => {
                    let removed = components.remove(&type_id);

                    if let Some(component) = removed {
                        if let Some(record) = self.get_component_record(id, type_id) {
                            record.removed(component);
                        }
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

    fn modify_entity(&mut self, id: EntityId, mut changes: impl Iterator<Item = ComponentChange>) {
        let curr_idx = self.archetypes.get_entity_archetype(id);

        if curr_idx.is_none() {
            return;
        }

        let curr_idx = curr_idx.unwrap();
        let hash_set: HashSet<TypeId> = self.archetypes[curr_idx].type_ids();

        let mut breaking_change = None;

        while let Some(change) = changes.next() {
            match change {
                ComponentChange::Added(component) => {
                    let type_id = (*component).as_any().type_id();

                    if hash_set.contains(&type_id) {
                        let removed = self.archetypes[curr_idx].replace_component(id, component);

                        if let Some(record) = self.get_component_record(id, type_id) {
                            record.added(removed);
                        }
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
            let bundle = self.archetypes[curr_idx].remove(id).unwrap();
            self.modify_bundle(id, bundle, std::iter::once(breaking_change).chain(changes));
        }
    }

    pub fn insert(&mut self, id: EntityId, components: impl Bundleable) {
        let changes = components
            .bundle()
            .into_iter()
            .map(|component| ComponentChange::Added(component));

        self.modify_entity(id, changes);
    }

    pub fn remove(&mut self, id: EntityId, types: &[TypeId]) {
        self.modify_entity(id, types.iter().cloned().map(ComponentChange::Removed));
    }

    pub fn get_component<T: 'static + Component>(&self, id: EntityId) -> Option<&T> {
        self.archetypes
            .get_entity_archetype(id)
            .and_then(|idx| self.archetypes[idx].get_component::<T>(id))
    }

    pub fn query<Fetch, Filter>(&self) -> Query<Fetch, Filter>
    where
        Fetch: 'static + QueryFetch,
        Filter: 'static + QueryFilter,
    {
        let archetypes = self.archetypes.get_query_archetypes::<Fetch, Filter>();
        Query::new(archetypes)
    }

    /// Tracks changes to the given component type. This must be called for query_changed to work.
    pub fn track_changes(&mut self, type_id: TypeId) {
        self.changes.entry(type_id).or_insert_with(SparseSet::new);
    }

    /// Returns a `QueryChanged` iterator for the given component type.
    /// If the component type is not being tracked, this will return `None`.
    pub fn query_changed<T: 'static + Component>(&self) -> Option<QueryChanged<T>> {
        let type_id = TypeId::of::<T>();

        if let Some(changes) = self.changes.get(&type_id) {
            Some(QueryChanged::new(&self, changes.iter()))
        } else {
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
            update.process(self);
        }

        for (_type_id, changes) in self.changes.iter_mut() {
            changes.clear();
        }
    }

    pub fn push_update(&self, update: WorldUpdate) {
        self.updates.borrow_mut().push(update);
    }
}
