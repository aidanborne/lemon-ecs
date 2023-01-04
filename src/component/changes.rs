use std::any::TypeId;

use super::Component;

pub enum ComponentChange {
    Added(Box<dyn Component>),
    Removed(TypeId),
}

#[derive(Default)]
pub struct ChangeRecord {
    added: bool,
    removed: Option<Box<dyn Component>>,
}

impl ChangeRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn added(&mut self, removed: Option<Box<dyn Component>>) {
        self.added = true;

        if self.removed.is_none() {
            self.removed = removed;
        }
    }

    pub fn was_added(&self) -> bool {
        self.added
    }

    pub fn removed(&mut self, removed: Box<dyn Component>) {
        self.added = false;

        if self.removed.is_none() {
            self.removed = Some(removed);
        }
    }

    pub fn get_removed(&self) -> Option<&Box<dyn Component>> {
        self.removed.as_ref()
    }
}
