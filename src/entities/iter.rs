use std::iter::Enumerate;

use crate::sparse_set;

use super::{Archetype, Entity, EntityId};

struct ArchetypeIter<'archetypes> {
    archetype: &'archetypes Archetype,
    entities: Enumerate<sparse_set::Iter<'archetypes, ()>>,
}

impl<'archetypes> Iterator for ArchetypeIter<'archetypes> {
    type Item = Entity<'archetypes>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.entities.next() {
            Some((idx, (id, _))) => Some(Entity {
                archetype: self.archetype,
                id: *id,
                idx,
            }),
            None => None,
        }
    }
}

pub(crate) struct Indices<'a, T> {
    values: &'a [T],
    indices: std::slice::Iter<'a, usize>,
}

impl<'a, T> Indices<'a, T> {
    pub fn new(values: &'a [T], indices: &'a [usize]) -> Self {
        Self {
            values,
            indices: indices.iter(),
        }
    }
}

impl<'a, T> Iterator for Indices<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.indices.next() {
            Some(idx) => Some(&self.values[*idx]),
            None => None,
        }
    }
}

macro_rules! impl_from_indices {
    ($ty:ident) => {
        impl<'archetypes> From<Indices<'archetypes, Archetype>> for $ty<'archetypes> {
            fn from(indices: Indices<'archetypes, Archetype>) -> Self {
                Self {
                    archetypes: indices,
                    iter: None,
                }
            }
        }
    };
}

pub struct EntityIter<'archetypes> {
    archetypes: Indices<'archetypes, Archetype>,
    iter: Option<ArchetypeIter<'archetypes>>,
}

impl<'archetype> Iterator for EntityIter<'archetype> {
    type Item = Entity<'archetype>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.as_mut() {
                Some(iter) => match iter.next() {
                    Some(entity) => return Some(entity),
                    None => self.iter = None,
                },
                None => match self.archetypes.next() {
                    Some(archetype) => {
                        self.iter = Some(ArchetypeIter {
                            archetype,
                            entities: archetype.entities().iter().enumerate(),
                        })
                    }
                    None => return None,
                },
            }
        }
    }
}

impl_from_indices!(EntityIter);

pub(crate) struct IdIter<'archetypes> {
    archetypes: Indices<'archetypes, Archetype>,
    iter: Option<sparse_set::Iter<'archetypes, ()>>,
}

impl<'archetype> Iterator for IdIter<'archetype> {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.as_mut() {
                Some(iter) => match iter.next() {
                    Some((id, _)) => return Some(*id),
                    None => self.iter = None,
                },
                None => match self.archetypes.next() {
                    Some(archetype) => self.iter = Some(archetype.entities().iter()),
                    None => return None,
                },
            }
        }
    }
}

impl_from_indices!(IdIter);
