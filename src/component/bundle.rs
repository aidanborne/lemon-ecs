use std::any::TypeId;

use lemon_ecs_macros::{all_tuples, impl_tuple_bundle};

use super::Component;

pub trait Bundle {
    fn components(self) -> Vec<Box<dyn Component>>;
}

impl<T: 'static + Component> Bundle for T {
    fn components(self) -> Vec<Box<dyn Component>> {
        vec![Box::new(self)]
    }
}

impl Bundle for Vec<Box<dyn Component>> {
    fn components(self) -> Vec<Box<dyn Component>> {
        self
    }
}

impl_tuple_bundle!(0..16);

pub trait TypeBundle {
    fn type_ids() -> Vec<TypeId>;
}

impl<T: 'static + Component> TypeBundle for T {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

macro_rules! impl_type_bundle {
    ($($t:ident),*) => {
        impl<$($t: TypeBundle),*> TypeBundle for ($($t,)*) {
            fn type_ids() -> Vec<TypeId> {
                (vec![$($t::type_ids()),*] as Vec<Vec<TypeId>>).concat()
            }
        }
    };
}

all_tuples!(impl_type_bundle, 0..16);
