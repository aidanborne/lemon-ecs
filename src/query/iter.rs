use std::marker::PhantomData;

use crate::entities::EntityIter;

use super::QueryRetriever;

pub struct QueryIter<'world, T: QueryRetriever> {
    entities: EntityIter<'world>,
    _marker: PhantomData<T>,
}

impl<'world, T: QueryRetriever> QueryIter<'world, T> {
    pub fn new(entities: EntityIter<'world>) -> Self {
        Self {
            entities,
            _marker: PhantomData,
        }
    }
}

impl<'world, T: QueryRetriever> Iterator for QueryIter<'world, T> {
    type Item = T::Output<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next().map(|entity| T::retrieve(&entity))
    }
}
