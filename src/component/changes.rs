use std::any::TypeId;

use super::Component;

pub enum ComponentChange {
    Added(Box<dyn Component>),
    Removed(TypeId),
}

#[derive(Default)]
pub enum ChangeRecord {
    /// Represents a change in the component.
    Changed(Box<dyn Component>),
    /// Represents a removal of the component.
    Removed(Box<dyn Component>),
    /// Represents an addition of the component.
    Added,
    /// Represents no change in the component.
    #[default]
    NoChange,
}

impl ChangeRecord {
    pub fn map_inserted(&mut self, replaced: Option<Box<dyn Component>>) {
        let old_value = std::mem::take(self);

        *self = match old_value {
            ChangeRecord::Changed(_) => old_value,
            ChangeRecord::Removed(original) => {
                assert!(replaced.is_none(), "Cannot replace a removed component");
                ChangeRecord::Changed(original)
            }
            ChangeRecord::Added => old_value,
            ChangeRecord::NoChange => {
                if let Some(original) = replaced {
                    // Component must have been replaced because it was present
                    ChangeRecord::Changed(original)
                } else {
                    ChangeRecord::Added
                }
            }
        }
    }

    pub fn map_removed(&mut self, removed: Box<dyn Component>) {
        let old_value = std::mem::take(self);

        *self = match old_value {
            ChangeRecord::Changed(original) => ChangeRecord::Removed(original),
            ChangeRecord::Removed(_) => old_value,
            ChangeRecord::Added => ChangeRecord::NoChange,
            ChangeRecord::NoChange => ChangeRecord::Removed(removed),
        }
    }

    pub fn is_added(&self) -> bool {
        matches!(self, ChangeRecord::Added)
    }

    pub fn is_changed(&self) -> bool {
        matches!(self, ChangeRecord::Changed(_))
    }

    pub fn is_removed(&self) -> bool {
        matches!(self, ChangeRecord::Removed(_))
    }

    pub fn is_no_change(&self) -> bool {
        matches!(self, ChangeRecord::NoChange)
    }

    pub fn get_removed(&self) -> Option<&dyn Component> {
        match self {
            ChangeRecord::Changed(original) => Some(&**original),
            ChangeRecord::Removed(original) => Some(&**original),
            _ => None,
        }
    }
}
