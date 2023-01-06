use std::marker::PhantomData;

use lemon_ecs_macros::all_tuples;

use crate::world::World;

mod params;
mod resource;

pub use params::SystemParameter;
pub use resource::*;

pub trait System {
    fn update(&self, world: &World);
}

pub trait IntoSystem<T> {
    fn into_system(self) -> Box<dyn System>;
}

/// Needed to implement `IntoSystem` for `Fn` items.
trait FnSystem<Args> {
    fn call(&self, world: &World);
}

struct CallableSystem<Func, Args> {
    f: Func,
    _args: PhantomData<Args>,
}

impl<F, Args> System for CallableSystem<F, Args>
where
    F: FnSystem<Args>,
{
    fn update(&self, world: &World) {
        self.f.call(world);
    }
}

impl<F, Args> IntoSystem<Args> for F
where
    F: FnSystem<Args> + 'static,
    Args: 'static,
{
    fn into_system(self) -> Box<dyn System> {
        Box::new(CallableSystem {
            f: self,
            _args: PhantomData::<Args>,
        })
    }
}

macro_rules! impl_system_fn {
    ($($param:ident),*) => {
        impl<$($param: SystemParameter,)* F: Fn($($param),*)> FnSystem<($($param,)*)> for F
            where F: Fn($(<$param as SystemParameter>::Result<'_>),*)
        {
            #[inline]
            fn call(&self, _world: &World) {
                self($($param::resolve(_world)),*);
            }
        }
    };
}

all_tuples!(impl_system_fn, 0..16);

pub mod prelude {
    pub use super::{params::SystemParameter, resource::*, System};
}
