use lemon_ecs_macros::impl_tuple_bundle;

use super::Component;

pub trait Bundleable {
    fn bundle(self) -> Vec<Box<dyn Component>>;
}

impl<T> Bundleable for T
where
    T: 'static + Component,
{
    fn bundle(self) -> Vec<Box<dyn Component>> {
        vec![Box::new(self)]
    }
}

impl Bundleable for Vec<Box<dyn Component>> {
    fn bundle(self) -> Vec<Box<dyn Component>> {
        self
    }
}

impl_tuple_bundle!(0..16);
