use std::{any::TypeId, collections::HashMap};

use crate::{component::Component, entities::EntityId};

use super::ChangeRecord;

#[derive(Default)]
#[repr(transparent)]
pub(crate) struct ChangeDetection {
    records: HashMap<TypeId, ChangeRecord>,
}

impl ChangeDetection {
    /// Returns a reference to the change record for the given component type.
    /// If the component type has not been registered, a new record is created.
    #[inline]
    fn get_mut_record(&mut self, type_id: TypeId) -> Option<&mut ChangeRecord> {
        self.records.get_mut(&type_id)
    }

    /// Consumes the change record for the given component type.
    pub fn consume_record<T: 'static + Component>(&mut self) -> Option<ChangeRecord> {
        let type_id = TypeId::of::<T>();

        if let Some(record) = self.records.get_mut(&type_id) {
            Some(std::mem::replace(record, ChangeRecord::from_type::<T>()))
        } else {
            self.records.insert(type_id, ChangeRecord::from_type::<T>());
            None
        }
    }

    /// Marks the component as added for the given entity.
    pub fn mark_added(&mut self, id: EntityId, type_id: TypeId) {
        if let Some(record) = self.get_mut_record(type_id) {
            record.mark_added(id);
        }
    }

    /// Marks the component as removed for the given entity.
    pub fn mark_removed(&mut self, id: EntityId, component: Box<dyn Component>) {
        if let Some(record) = self.get_mut_record((*component).as_any().type_id()) {
            record.mark_removed(id, component);
        }
    }

    /// Marks the component as changed for the given entity.
    pub fn mark_changed(&mut self, id: EntityId, component: Box<dyn Component>) {
        if let Some(record) = self.get_mut_record((*component).as_any().type_id()) {
            record.mark_changed(id, component);
        }
    }

    /// Indicates that a system has finished processing the given entity.
    pub fn contains(&self, id: EntityId) -> bool {
        self.records.values().any(|record| record.contains(id))
    }
}
