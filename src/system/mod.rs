use std::marker::PhantomData;

use lemon_ecs_macros::all_tuples;

use crate::world::World;

use self::params::SystemParameter;

pub mod buffer;
pub mod params;
pub mod resource;

pub trait System {
    fn update<'a>(&self, world: &'a World);
}

pub type BoxedSystem = Box<dyn System>;

pub trait IntoSystem<T> {
    fn into_system(self) -> BoxedSystem;
}

/// Needed to implement `IntoSystem` for `Fn` items.
trait SystemFn<Args> {
    fn call<'a>(&self, world: &'a World);
}

struct SystemImpl<Func, Args> {
    f: Func,
    _args: PhantomData<Args>,
}

impl<F, Args> System for SystemImpl<F, Args>
where
    F: SystemFn<Args>,
{
    fn update(&self, world: &World) {
        self.f.call(world);
    }
}

impl<F, Args> IntoSystem<Args> for F
where
    F: SystemFn<Args> + 'static,
    Args: 'static,
{
    fn into_system(self) -> BoxedSystem {
        Box::new(SystemImpl {
            f: self,
            _args: PhantomData::<Args>,
        })
    }
}

macro_rules! impl_system_fn {
    ($($param:ident),*) => {
        impl<$($param: SystemParameter,)* F: Fn($($param),*)> SystemFn<($($param,)*)> for F
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
