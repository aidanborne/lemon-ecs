use crate::{
    query::{fetch::QueryFetch, filter::QueryFilter, Query},
    world::World,
};

pub trait SystemParameter {
    type Result<'a>;

    fn resolve<'a>(world: &'a World) -> Self::Result<'a>;
}

impl<Fetch: 'static + QueryFetch, Filter: QueryFilter + 'static> SystemParameter
    for Query<'_, Fetch, Filter>
{
    type Result<'a> = Query<'a, Fetch, Filter>;

    fn resolve<'a>(world: &'a World) -> Self::Result<'a> {
        world.query::<Query<'a, Fetch, Filter>>()
    }
}

/*impl SystemParameter for &'_ World {
    type Item<'a> = &'a World;

    fn get_value<'a>(world: &'a World) -> Self::Item<'a> {
        world
    }
}*/
