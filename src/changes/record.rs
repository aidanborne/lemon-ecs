use crate::component::Component;

#[derive(Default)]
pub enum ChangeRecord {
    /// The component was changed.
    Changed(Box<dyn Component>),
    /// The component was removed.
    Removed(Box<dyn Component>),
    /// The component was added.
    Added,
    /// The component was not changed.
    #[default]
    Unchanged,
}

impl ChangeRecord {
    pub fn map_insertion(&mut self, replaced: Option<Box<dyn Component>>) {
        let old_record = std::mem::replace(self, ChangeRecord::Unchanged);

        *self = match old_record {
            ChangeRecord::Removed(original) => {
                assert!(replaced.is_none(), "Cannot replace a removed component");
                ChangeRecord::Changed(original)
            }
            ChangeRecord::Unchanged => {
                if let Some(original) = replaced {
                    // Component must have been replaced because it was present
                    ChangeRecord::Changed(original)
                } else {
                    ChangeRecord::Added
                }
            }
            _ => old_record,
        }
    }

    pub fn map_removal(&mut self, removed: Box<dyn Component>) {
        let old_record = std::mem::replace(self, ChangeRecord::Unchanged);

        *self = match old_record {
            ChangeRecord::Changed(original) => ChangeRecord::Removed(original),
            ChangeRecord::Removed(_) => old_record,
            ChangeRecord::Added => ChangeRecord::Unchanged,
            ChangeRecord::Unchanged => ChangeRecord::Removed(removed),
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
        matches!(self, ChangeRecord::Unchanged)
    }

    pub fn get_removed(&self) -> Option<&dyn Component> {
        match self {
            ChangeRecord::Changed(original) | ChangeRecord::Removed(original) => Some(&**original),
            _ => None,
        }
    }
}
