use std::any::TypeId;

use super::Component;

pub enum ComponentChange {
    Added(Box<dyn Component>),
    Removed(TypeId),
}

#[derive(Default)]
pub enum ChangeRecord {
    Added(Option<Box<dyn Component>>),
    Removed(Box<dyn Component>),
    #[default]
    NoChange,
}

impl ChangeRecord {
    pub fn map_added(&mut self, replaced: Option<Box<dyn Component>>) {
        let old_value = std::mem::take(self);

        *self = match old_value {
            ChangeRecord::NoChange => ChangeRecord::Added(replaced),
            ChangeRecord::Removed(original) => ChangeRecord::Added(Some(original)),
            _ => old_value,
        }
    }

    pub fn map_removed(&mut self, removed: Box<dyn Component>) {
        let old_value = std::mem::take(self);

        *self = match old_value {
            ChangeRecord::NoChange => ChangeRecord::Removed(removed),
            ChangeRecord::Added(original) => ChangeRecord::Removed(original.unwrap_or(removed)),
            _ => old_value,
        }
    }

    pub fn is_added(&self) -> bool {
        matches!(self, ChangeRecord::Added(_))
    }

    pub fn is_removed(&self) -> bool {
        matches!(self, ChangeRecord::Removed(_))
    }

    pub fn is_no_change(&self) -> bool {
        matches!(self, ChangeRecord::NoChange)
    }

    pub fn get_removed(&self) -> Option<&dyn Component> {
        match self {
            ChangeRecord::Added(component) => component.as_ref().map(|component| &**component),
            ChangeRecord::Removed(component) => Some(&**component),
            _ => None,
        }
    }
}
