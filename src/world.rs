use std::any::TypeId;

use crate::{
    component::Component,
    query::{Query, Queryable},
    storage::{archetypes::ArchetypeArena, bundle::ComponentBundle},
};

pub struct World {
    available_ids: Vec<usize>,
    next_id: usize,
    archetypes: ArchetypeArena,
}

impl World {
    pub fn new() -> Self {
        Self {
            available_ids: Vec::new(),
            next_id: 0,
            archetypes: Default::default(),
        }
    }

    pub fn spawn(&mut self) -> usize {
        let id = match self.available_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 1;
                id
            }
        };

        let bundle = ComponentBundle::new();

        let idx = self.archetypes.get_bundle_archetype(&bundle);

        self.archetypes[idx].insert(id, bundle);
        id
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
                self.archetypes[curr_idx].replace(id, component);
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

    pub fn query<'a, T: Queryable<'a>>(&'a self) -> Query<'a, T::Fetch, T::Filter> {
        Query::new(self.archetypes.query_archetypes::<T>())
    }
}
