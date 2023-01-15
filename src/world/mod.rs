use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use crate::{
    component::{Bundle, Component, ComponentChange, TypeBundle},
    entities::{Archetypes, Entities, EntityId},
    query::{Query, QueryChanged, QueryFetch, QueryFilter},
    system::Resource,
    traits::Downcast,
};

mod buffer;
mod changes;
mod updates;

pub use buffer::*;
pub(crate) use updates::*;

use changes::Changes;

#[derive(Default)]
pub struct World {
    entities: Entities,
    archetypes: Archetypes,
    updates: RefCell<Vec<WorldUpdate>>,
    resources: HashMap<TypeId, Box<dyn Resource>>,
    changes: Changes,
    despawned: Vec<EntityId>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, components: impl Bundle) -> EntityId {
        let id = self.entities.spawn().into();

        let bundle = components.components();

        self.archetypes
            .component_archetype(&bundle)
            .insert(id, bundle);
        id
    }

    pub fn despawn(&mut self, id: EntityId) {
        if let Some(archetype) = self.archetypes.entity_archetype_mut(id) {
            if let Some(components) = archetype.remove(id) {
                for component in components {
                    let type_id = (*component).as_any().type_id();

                    if let Some(record) = self.changes.get_record(id, type_id) {
                        record.map_removed(component);
                    }
                }

                if self.changes.is_processed(id) {
                    self.entities.despawn(*id);
                } else {
                    self.despawned.push(id);
                }
            } else {
                self.entities.despawn(*id);
            }
        }
    }

    pub fn has_component<T: 'static + Component>(&self, id: EntityId) -> bool {
        self.archetypes
            .entity_archetype(id)
            .map(|archetype| archetype.has_component(TypeId::of::<T>()))
            .unwrap_or(false)
    }

    #[inline]
    fn modify_bundle<Iter>(&mut self, id: EntityId, bundle: Vec<Box<dyn Component>>, changes: Iter)
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

                    if let Some(record) = self.changes.get_record(id, type_id) {
                        record.map_inserted(removed);
                    }
                }
                ComponentChange::Removed(type_id) => {
                    let removed = components.remove(&type_id);

                    if let Some(component) = removed {
                        if let Some(record) = self.changes.get_record(id, type_id) {
                            record.map_removed(component);
                        }
                    }
                }
            }
        }

        let components: Vec<_> = components.into_values().collect();

        self.archetypes
            .component_archetype(&components)
            .insert(id, components);
    }

    fn modify_entity(&mut self, id: EntityId, mut changes: impl Iterator<Item = ComponentChange>) {
        let archetype_idx = self.archetypes.entity_archetype_idx(id);

        if archetype_idx.is_none() {
            return;
        }

        let archetype_idx = archetype_idx.unwrap();
        let hash_set: HashSet<TypeId> = self.archetypes[archetype_idx].type_ids();

        let mut consumed = None;

        for change in changes.by_ref() {
            match change {
                ComponentChange::Added(component) => {
                    let type_id = (*component).as_any().type_id();

                    if hash_set.contains(&type_id) {
                        let removed =
                            self.archetypes[archetype_idx].replace_component(id, component);

                        if let Some(record) = self.changes.get_record(id, type_id) {
                            record.map_inserted(removed);
                        }
                    } else {
                        consumed = Some(ComponentChange::Added(component));
                        break;
                    }
                }
                ComponentChange::Removed(type_id) => {
                    if hash_set.contains(&type_id) {
                        consumed = Some(change);
                        break;
                    }
                }
            }
        }

        if let Some(consumed) = consumed {
            let bundle = self.archetypes[archetype_idx].remove(id).unwrap();
            self.modify_bundle(id, bundle, std::iter::once(consumed).chain(changes));
        }
    }

    pub fn insert(&mut self, id: EntityId, components: impl Bundle) {
        let changes = components
            .components()
            .into_iter()
            .map(|component| ComponentChange::Added(component));

        self.modify_entity(id, changes);
    }

    pub fn remove<T: TypeBundle>(&mut self, id: EntityId) {
        self.modify_entity(id, T::type_ids().into_iter().map(ComponentChange::Removed));
    }

    pub fn get_component<T: 'static + Component>(&self, id: EntityId) -> Option<&T> {
        self.archetypes
            .entity_archetype(id)
            .and_then(|archetype| archetype.get_component::<T>(id))
    }

    pub fn query<Fetch, Filter>(&self) -> Query<Fetch, Filter>
    where
        Fetch: 'static + QueryFetch,
        Filter: 'static + QueryFilter,
    {
        let archetypes = self.archetypes.query_archetypes::<Fetch, Filter>();
        Query::new(self, archetypes)
    }

    /// Returns a `QueryChanged` iterator for the given component type.
    pub fn query_changed<T: 'static + Component>(&self) -> QueryChanged<T> {
        let sparse_set = self.changes.get_sparse_set(TypeId::of::<T>());
        QueryChanged::new(self, sparse_set.iter())
    }

    pub fn get_resource<T: 'static + Resource>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|resource| resource.downcast_ref::<T>().unwrap())
    }

    pub fn get_resource_mut<T: 'static + Resource>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .map(|resource| resource.downcast_mut::<T>().unwrap())
    }

    pub fn insert_resource<T: 'static + Resource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub(crate) fn process_updates(&mut self) {
        let updates = std::mem::take(&mut *self.updates.borrow_mut());
        WorldUpdate::process(self, updates);

        self.changes.clear_processed();

        self.despawned.retain(|&id| {
            if self.changes.is_processed(id) {
                self.entities.despawn(*id);
                false
            } else {
                true
            }
        });

        for resource in self.resources.values_mut() {
            resource.update();
        }
    }

    pub(crate) fn push_update(&self, update: WorldUpdate) {
        self.updates.borrow_mut().push(update);
    }
}
