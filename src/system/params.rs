use crate::{
    query::{Query, QueryFetch, QueryFilter},
    world::{World, WorldBuffer},
};

pub trait SystemParameter {
    type Output<'world>;

    fn resolve(world: &World) -> Self::Output<'_>;
}

impl<Fetch: 'static + QueryFetch, Filter: 'static + QueryFilter> SystemParameter
    for Query<'_, Fetch, Filter>
{
    type Output<'world> = Query<'world, Fetch, Filter>;

    fn resolve(world: &World) -> Self::Output<'_> {
        world.query::<Fetch, Filter>()
    }
}

/*impl<T: 'static + Component> SystemParameter for QueryChanged<'_, T> {
    type Output<'world> = QueryChanged<'world, T>;

    fn resolve(world: &World) -> Self::Output<'_> {
        world.query_changed::<T>()
    }
}*/

impl SystemParameter for &'_ World {
    type Output<'world> = &'world World;

    fn resolve(world: &World) -> Self::Output<'_> {
        world
    }
}

impl SystemParameter for WorldBuffer<'_> {
    type Output<'world> = WorldBuffer<'world>;

    fn resolve(world: &World) -> Self::Output<'_> {
        WorldBuffer::new(world)
    }
}
