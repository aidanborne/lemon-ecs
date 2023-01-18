use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use crate::{
    changes::{ChangeDetection, ComponentChange},
    component::{Bundle, Component, TypeBundle},
    entities::{Archetypes, Entities, EntityId, EntityIter},
    query::{Query, QueryChanged, QueryRetriever, QuerySelector},
};

mod buffer;
mod updates;

pub use buffer::*;
pub(crate) use updates::*;

#[derive(Default)]
pub struct World {
    entities: Entities,
    archetypes: Archetypes,
    updates: RefCell<Vec<WorldUpdate>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
    changes: ChangeDetection,
    despawned: Vec<EntityId>,
}

impl World {
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
                    self.changes.mark_removed(id, component);
                }

                if !self.changes.contains(id) {
                    self.despawned.push(id);
                } else {
                    self.entities.despawn(*id);
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

                    if let Some(changed) = components.insert(type_id, component) {
                        self.changes.mark_changed(id, changed);
                    } else {
                        self.changes.mark_added(id, type_id);
                    }
                }
                ComponentChange::Removed(type_id) => {
                    let removed = components.remove(&type_id);

                    if let Some(component) = removed {
                        self.changes.mark_removed(id, component);
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
        let archetype = self.archetypes.entity_archetype_mut(id);

        if archetype.is_none() {
            return;
        }

        let archetype = archetype.unwrap();
        let hash_set: HashSet<TypeId> = archetype.type_ids(); //self.archetypes[archetype_idx].type_ids();

        let mut consumed = None;

        for change in changes.by_ref() {
            match change {
                ComponentChange::Added(component) => {
                    let type_id = (*component).as_any().type_id();

                    if hash_set.contains(&type_id) {
                        let changed = archetype.replace_component(id, component);

                        self.changes.mark_changed(id, changed);
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
            let bundle = archetype.remove(id).unwrap();
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

    pub fn query<T>(&mut self) -> Query<T>
    where
        T: 'static + QueryRetriever,
    {
        Query::new(self)
    }

    pub fn query_selector<T>(&mut self) -> EntityIter
    where
        T: 'static + QuerySelector,
    {
        self.archetypes.query_entities::<T>()
    }

    /// Returns a `QueryChanged` iterator for the given component type.
    pub fn query_changed<T: 'static + Component>(&mut self) -> QueryChanged<T> {
        let record = self.changes.consume_record::<T>();
        QueryChanged::new(self, record)
    }

    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|resource| resource.downcast_ref::<T>().unwrap())
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .map(|resource| resource.downcast_mut::<T>().unwrap())
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub(crate) fn process_updates(&mut self) {
        let updates = std::mem::take(&mut *self.updates.borrow_mut());
        WorldUpdate::process(self, updates);

        self.despawned.retain(|&id| {
            if self.changes.contains(id) {
                self.entities.despawn(*id);
                false
            } else {
                true
            }
        });
    }

    pub(crate) fn push_update(&self, update: WorldUpdate) {
        self.updates.borrow_mut().push(update);
    }
}
