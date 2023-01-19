use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use crate::{
    buffer::WorldBuffer,
    changes::{ChangeDetection, ChangeRecord},
    component::{Bundle, Component, TypeBundle},
    entities::{Archetypes, Entities, EntityId, EntityIter},
    query::{Query, QueryChanged, QueryRetriever, QuerySelector},
};

pub enum ComponentChange {
    Insert(Box<dyn Component>),
    Remove(TypeId),
}

#[derive(Default)]
pub struct World {
    entities: Entities,
    archetypes: Archetypes,
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
    fn modify_bundle<T>(&mut self, id: EntityId, bundle: Vec<Box<dyn Component>>, changes: T)
    where
        T: Iterator<Item = ComponentChange>,
    {
        let mut components: HashMap<TypeId, Box<dyn Component>> = bundle
            .into_iter()
            .map(|component| ((*component).as_any().type_id(), component))
            .collect();

        for change in changes {
            match change {
                ComponentChange::Insert(component) => {
                    let type_id = (*component).as_any().type_id();

                    if let Some(changed) = components.insert(type_id, component) {
                        self.changes.mark_changed(id, changed);
                    } else {
                        self.changes.mark_added(id, type_id);
                    }
                }
                ComponentChange::Remove(type_id) => {
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

    pub fn modify(&mut self, id: EntityId, changes: impl IntoIterator<Item = ComponentChange>) {
        let archetype = self.archetypes.entity_archetype_mut(id);

        if archetype.is_none() {
            return;
        }

        let archetype = archetype.unwrap();
        let hash_set: HashSet<TypeId> = archetype.type_ids();

        let mut consumed = None;

        let changes = &mut changes.into_iter();

        for change in changes.by_ref() {
            match change {
                ComponentChange::Insert(component) => {
                    let type_id = (*component).as_any().type_id();

                    if hash_set.contains(&type_id) {
                        let changed = archetype.replace_component(id, component);

                        self.changes.mark_changed(id, changed);
                    } else {
                        consumed = Some(ComponentChange::Insert(component));
                        break;
                    }
                }
                ComponentChange::Remove(type_id) => {
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
            .map(|component| ComponentChange::Insert(component));

        self.modify(id, changes);
    }

    pub fn remove<T: TypeBundle>(&mut self, id: EntityId) {
        self.modify(id, T::type_ids().into_iter().map(ComponentChange::Remove));
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

    /// Returns a query for all entities that have changed since the last call.
    /// Entities will not be despawned until their changes have been processed.
    pub fn query_changed<T: 'static + Component>(&mut self) -> QueryChanged<T> {
        let record = self
            .changes
            .consume_record::<T>()
            .unwrap_or_else(|| ChangeRecord::from_ids::<T>(self.archetypes.query_ids::<T>()));

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

    pub(crate) fn insert_resource_boxed(&mut self, resource: Box<dyn Any>) {
        self.resources.insert((*resource).type_id(), resource);
    }

    #[inline]
    pub fn apply_buffer(&mut self, buffer: WorldBuffer) {
        buffer.apply_world(self);
    }
}
