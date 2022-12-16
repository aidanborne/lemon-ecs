use std::{any::TypeId, collections::HashSet};

pub mod queryable;
pub mod result;

#[derive(PartialEq, Clone, Copy)]
pub enum QueryComparison {
    Exact,
    Partial,
    None,
}

impl QueryComparison {
    pub fn is_exact(&self) -> bool {
        matches!(self, QueryComparison::Exact)
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, QueryComparison::None)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Query(Vec<TypeId>);

impl Query {
    pub fn new(type_ids: Vec<TypeId>) -> Self {
        Self(type_ids)
    }

    pub fn normalize(&self) -> Self {
        let mut type_ids = self.0.clone();
        type_ids.sort();
        type_ids.dedup();
        Self::new(type_ids)
    }

    pub fn compare_to(&self, other: &Query) -> QueryComparison {
        let mut set: HashSet<&TypeId> = other.0.iter().collect();

        for type_id in &self.0 {
            if !set.contains(type_id) {
                return QueryComparison::None;
            }

            set.remove(type_id);
        }

        if set.is_empty() {
            QueryComparison::Exact
        } else {
            QueryComparison::Partial
        }
    }
}
