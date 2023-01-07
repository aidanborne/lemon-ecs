use crate::{
    component::Component,
    query::{Query, QueryChanged, QueryFetch, QueryFilter},
    world::{World, WorldBuffer},
};

pub trait SystemParameter {
    type Result<'world>;

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

    fn resolve(world: &World) -> Self::Result<'_> {
        world.query_changed::<T>()
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
