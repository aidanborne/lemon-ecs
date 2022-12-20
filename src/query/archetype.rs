use std::{any::TypeId, hash::Hash, ops::Deref};

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Archetype {
    type_ids: Vec<TypeId>,
}

impl Archetype {
    pub fn new(mut type_ids: Vec<TypeId>) -> Self {
        type_ids.sort_unstable();
        type_ids.dedup();
        Self { type_ids }
    }
}

impl Deref for Archetype {
    type Target = Vec<TypeId>;

    fn deref(&self) -> &Self::Target {
        &self.type_ids
    }
}

impl FromIterator<TypeId> for Archetype {
    fn from_iter<T: IntoIterator<Item = TypeId>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
