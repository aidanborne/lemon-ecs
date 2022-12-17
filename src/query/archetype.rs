use std::{any::TypeId, collections::HashSet};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Matching {
    Exact,
    Partial,
    None,
}

impl Matching {
    pub fn is_exact(&self) -> bool {
        matches!(self, Matching::Exact)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Matching::Partial)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Matching::None)
    }
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Archetype(Vec<TypeId>);

impl Archetype {
    pub fn new(type_ids: Vec<TypeId>) -> Self {
        Self(type_ids)
    }

    pub fn matches(&self, other: &Archetype) -> Matching {
        let mut set: HashSet<&TypeId> = other.0.iter().collect();

        for type_id in &self.0 {
            if !set.contains(type_id) {
                return Matching::None;
            }

            set.remove(type_id);
        }

        if set.is_empty() {
            Matching::Exact
        } else {
            Matching::Partial
        }
    }
}

impl FromIterator<TypeId> for Archetype {
    fn from_iter<T: IntoIterator<Item = TypeId>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
