use std::{any::TypeId, cell::RefCell, collections::HashMap};

use crate::{component::ChangeRecord, storage::sparse_set::SparseSet};

use super::EntityId;

#[derive(PartialEq)]
enum ChangeStatus {
    Processed,
    Unprocessed,
}

pub(crate) struct Changes {
    empty_sparse_set: SparseSet<ChangeRecord>,
    tracked_changes: RefCell<HashMap<TypeId, ChangeStatus>>,
    changes: HashMap<TypeId, SparseSet<ChangeRecord>>,
}

impl Changes {
    pub fn new() -> Self {
        Self {
            empty_sparse_set: SparseSet::new(),
            tracked_changes: RefCell::default(),
            changes: HashMap::new(),
        }
    }

    pub fn get_record(&mut self, id: EntityId, type_id: TypeId) -> Option<&mut ChangeRecord> {
        if self.tracked_changes.borrow().contains_key(&type_id) {
            let sparse_set = self.changes.entry(type_id).or_insert_with(SparseSet::new);
            return Some(sparse_set.get_or_insert_with(*id, ChangeRecord::default));
        }

        None
    }

    fn set_status(&self, type_id: TypeId, status: ChangeStatus) {
        self.tracked_changes.borrow_mut().insert(type_id, status);
    }

    pub fn get_sparse_set(&self, type_id: TypeId) -> &SparseSet<ChangeRecord> {
        let sparse_set = self.changes.get(&type_id);

        match sparse_set {
            Some(sparse_set) => {
                self.set_status(type_id, ChangeStatus::Processed);
                sparse_set
            }
            None => {
                self.set_status(type_id, ChangeStatus::Unprocessed);
                &self.empty_sparse_set
            }
        }
    }

    pub fn clear_processed(&mut self) {
        let mut tracked_changes = self.tracked_changes.borrow_mut();

        for (type_id, status) in tracked_changes.iter() {
            if *status == ChangeStatus::Processed {
                self.changes.remove(type_id);
            }
        }

        tracked_changes.clear();
    }
}

impl Default for Changes {
    fn default() -> Self {
        Self::new()
    }
}
