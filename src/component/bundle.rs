use lemon_ecs_macros::impl_tuple_bundle;

use super::Component;

pub type ComponentBundle = Vec<Box<dyn Component>>;

pub trait Bundleable {
    fn bundle(self) -> ComponentBundle;
}

impl<T> Bundleable for T
where
    T: 'static + Component,
{
    fn bundle(self) -> ComponentBundle {
        vec![Box::new(self)]
    }
}

impl Bundleable for ComponentBundle {
    fn bundle(self) -> ComponentBundle {
        self
    }
}

impl_tuple_bundle!(0..16);
