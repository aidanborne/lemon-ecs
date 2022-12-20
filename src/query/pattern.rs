use std::{any::TypeId, collections::HashSet};

use super::{archetype::Archetype, filter::Filter};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct QueryPattern {
    type_ids: Vec<TypeId>,
    filters: Vec<Filter>,
}

impl QueryPattern {
    pub fn new(mut type_ids: Vec<TypeId>, mut filters: Vec<Filter>) -> Self {
        type_ids.sort_unstable();
        type_ids.dedup();

        filters.sort_unstable();
        filters.dedup();

        Self { type_ids, filters }
    }

    pub fn filter(&self, archetype: &Archetype) -> bool {
        let hash_set = archetype.iter().collect::<HashSet<_>>();

        for type_id in self.type_ids.iter() {
            if !hash_set.contains(type_id) {
                return false;
            }
        }

        for filter in self.filters.iter() {
            match filter {
                Filter::With(type_id) => {
                    if !hash_set.contains(type_id) {
                        return false;
                    }
                }
                Filter::Without(type_id) => {
                    if hash_set.contains(type_id) {
                        return false;
                    }
                }
            }
        }

        true
    }
}
