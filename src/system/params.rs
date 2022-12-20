use crate::{
    query::{fetch::QueryFetch, filter::QueryFilter, Query},
    world::World,
};

pub trait SystemParameter {
    type Item<'a>;

    fn get_value<'a>(world: &'a World) -> Self::Item<'a>;
}

impl<Fetch: 'static + QueryFetch, Filter: QueryFilter + 'static> SystemParameter
    for Query<'_, Fetch, Filter>
{
    type Item<'a> = Query<'a, Fetch, Filter>;

    fn get_value<'a>(world: &'a World) -> Self::Item<'a> {
        world.query::<Query<'a, Fetch, Filter>>()
    }
}

/*impl SystemParameter for &'_ World {
    type Item<'a> = &'a World;

    fn get_value<'a>(world: &'a World) -> Self::Item<'a> {
        world
    }
}*/

pub trait SystemArguments {
    fn get_result<'a>(world: &'a World) -> Self;
}

/*macro_rules! impl_system_arguments {
    ($($t:ident),*) => {
        impl<$($t: 'static + for<'a> SystemParameter<'a>),*> SystemArguments for ($($t,)*)
        {
            fn get_result<'a>(_world: &'a World) -> Self {
                ($($t::get_value(_world),)*)
            }
        }
    };
}

all_tuples!(impl_system_arguments, 0..16);*/
