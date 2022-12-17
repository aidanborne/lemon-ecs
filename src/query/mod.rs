use std::{iter::Peekable, marker::PhantomData};

use crate::storage::{entities::EntityStorage, sparse_set};

mod archetype;
mod queryable;

pub use queryable::*;
pub use archetype::*;

pub struct Query<'a, T: Queryable<'a>> {
    archetypes: &'a [EntityStorage],
    indices: Peekable<std::vec::IntoIter<usize>>,
    entities: Option<sparse_set::Keys<'a, usize>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Queryable<'a>> Query<'a, T> {
    pub fn new(archetypes: &'a Vec<EntityStorage>, indices: Vec<usize>) -> Self {
        Self {
            archetypes,
            indices: indices.into_iter().peekable(),
            entities: None,
            _marker: PhantomData,
        }
    }

    fn peek_archetype(&mut self) -> Option<&'a EntityStorage> {
        self.indices.peek().map(|idx| &self.archetypes[*idx])
    }

    fn next_entity(&mut self) -> Option<usize> {
        loop {
            if let Some(entities) = &mut self.entities {
                if let Some(id) = entities.next() {
                    return Some(id);
                }

                self.indices.next();
            }

            if let Some(archetype) = self.peek_archetype() {
                self.entities = Some(archetype.iter());
            } else {
                return None;
            }
        }
    }
}

impl<'a, T: Queryable<'a>> Iterator for Query<'a, T> {
    type Item = T::Result;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entity()
            .map(|id| T::map_entity(self.peek_archetype().unwrap(), id))
    }
}
