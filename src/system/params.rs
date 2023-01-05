use std::any::TypeId;

use crate::{
    component::Component,
    query::{fetch::QueryFetch, filter::QueryFilter, Query, QueryChanged},
    world::{buffer::WorldBuffer, World},
};

pub trait SystemParameter {
    type Result<'world>;

    fn init(_world: &mut World) {}

    fn resolve(world: &World) -> Self::Result<'_>;
}

impl<Fetch: 'static + QueryFetch, Filter: 'static + QueryFilter> SystemParameter
    for Query<'_, Fetch, Filter>
{
    type Result<'world> = Query<'world, Fetch, Filter>;

    fn resolve(world: &World) -> Self::Result<'_> {
        world.query::<Fetch, Filter>()
    }
}

impl<T: 'static + Component> SystemParameter for QueryChanged<'_, T> {
    type Result<'world> = QueryChanged<'world, T>;

    fn init(world: &mut World) {
        world.track_changes(TypeId::of::<T>());
    }

    fn resolve(world: &World) -> Self::Result<'_> {
        world.query_changed::<T>().unwrap()
    }
}

impl SystemParameter for &'_ World {
    type Result<'world> = &'world World;

    fn resolve(world: &World) -> Self::Result<'_> {
        world
    }
}

impl SystemParameter for WorldBuffer<'_> {
    type Result<'world> = WorldBuffer<'world>;

    fn resolve(world: &World) -> Self::Result<'_> {
        WorldBuffer::new(world)
    }
}
