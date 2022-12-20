use std::{iter::Peekable, marker::PhantomData};

use crate::storage::{
    archetypes,
    entities::{self, EntityStorage},
};

use self::{fetch::QueryFetch, filter::QueryFilter, pattern::QueryPattern};

pub mod archetype;
pub mod fetch;
pub mod filter;
pub mod pattern;

pub struct Query<'a, Fetch: QueryFetch<'a> + 'a, Filter: QueryFilter + 'a = ()> {
    archetypes: Peekable<archetypes::Iter<'a>>,
    entities: Option<entities::Iter<'a>>,
    _fetch: PhantomData<Fetch>,
    _filter: PhantomData<Filter>,
}

impl<'a, Fetch: QueryFetch<'a>, Filter: QueryFilter> Query<'a, Fetch, Filter> {
    pub fn new(archetypes: archetypes::Iter<'a>) -> Self {
        Self {
            archetypes: archetypes.peekable(),
            entities: None,
            _fetch: PhantomData,
            _filter: PhantomData,
        }
    }

    fn peek_archetype(&mut self) -> Option<&'a EntityStorage> {
        self.archetypes.peek().copied()
    }

    fn next_entity(&mut self) -> Option<entities::Entity> {
        loop {
            if let Some(entities) = &mut self.entities {
                if let Some(entry) = entities.next() {
                    return Some(entry);
                }

                self.archetypes.next();
            }

            if let Some(archetype) = self.peek_archetype() {
                self.entities = Some(archetype.iter());
            } else {
                return None;
            }
        }
    }
}

impl<'a, Fetch: QueryFetch<'a>, Filter: QueryFilter> Iterator for Query<'a, Fetch, Filter> {
    type Item = Fetch::Result;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entity()
            .map(|entity| Fetch::get_result(entity, self.peek_archetype().unwrap()))
    }
}

pub trait Queryable<'a> {
    type Fetch: QueryFetch<'a> + 'static;
    type Filter: QueryFilter + 'static;

    fn get_pattern() -> QueryPattern {
        QueryPattern::new(Self::Fetch::get_type_ids(), Self::Filter::get_filters())
    }
}

impl<'a, T: QueryFetch<'a> + 'static> Queryable<'a> for T {
    type Fetch = Self;
    type Filter = ();
}

impl<'a, Fetch: QueryFetch<'a> + 'static, Filter: QueryFilter + 'static> Queryable<'a>
    for Query<'a, Fetch, Filter>
{
    type Fetch = Fetch;
    type Filter = Filter;
}
