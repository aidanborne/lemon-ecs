use crate::{
    component::{Component, ComponentVec},
    entities::EntityId,
    sparse_set::SparseSet,
};

pub(crate) enum ChangeStatus {
    Added,
    Modified(usize),
    Removed(usize),
}

/// Records changes to a specific component type.
/// Although it is untyped, it is only usable for a single component type.
pub(crate) struct ChangeRecord {
    pub entities: SparseSet<ChangeStatus>,
    pub removed: Box<dyn ComponentVec>,
}

impl ChangeRecord {
    /*pub fn from_component(component: &dyn Component) -> Self {
        Self {
            entities: SparseSet::new(),
            removed: component.as_empty_vec(),
        }
    }*/

    pub fn from_type<T: Component>() -> Self {
        Self {
            entities: SparseSet::new(),
            removed: Box::<Vec<T>>::default(),
        }
    }

    pub fn from_ids<T: Component>(ids: impl Iterator<Item = EntityId>) -> Self {
        Self {
            entities: SparseSet::from_iter(ids.map(|id| (id, ChangeStatus::Added))),
            removed: Box::<Vec<T>>::default(),
        }
    }

    /// Marks the component as added for the given entity.
    pub fn mark_added(&mut self, id: EntityId) {
        match self.entities.get_mut(id) {
            Some(status) => match status {
                ChangeStatus::Removed(id) => *status = ChangeStatus::Modified(*id),
                _ => panic!("Cannot add a component that was already added."),
            },
            None => {
                self.entities.insert(id, ChangeStatus::Added);
            }
        }
    }

    /// Marks the component as removed for the given entity.
    pub fn mark_removed(&mut self, id: EntityId, removed: Box<dyn Component>) {
        match self.entities.get_mut(id) {
            Some(status) => match status {
                ChangeStatus::Added => {
                    self.entities.remove(id);
                }
                ChangeStatus::Modified(id) => {
                    self.removed.replace(*id, removed);
                    *status = ChangeStatus::Removed(*id);
                }
                ChangeStatus::Removed(_) => {
                    panic!("Cannot remove a component that was already removed")
                }
            },
            None => {
                self.entities
                    .insert(id, ChangeStatus::Removed(self.removed.push(removed)));
            }
        }
    }

    /// Marks the component as changed for the given entity.
    pub fn mark_changed(&mut self, id: EntityId, removed: Box<dyn Component>) {
        match self.entities.get_mut(id) {
            Some(status) => match status {
                ChangeStatus::Added | ChangeStatus::Modified(_) => {}
                ChangeStatus::Removed(_) => {
                    panic!("Cannot change a component that was already removed")
                }
            },
            None => {
                self.entities
                    .insert(id, ChangeStatus::Modified(self.removed.push(removed)));
            }
        }
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains(id)
    }
}
