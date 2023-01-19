use std::{any::TypeId, marker::PhantomData};

use crate::entities::Archetype;

use super::QuerySelector;

pub struct Without<T>(PhantomData<T>);

impl<T: 'static> QuerySelector for Without<T> {
    fn filter(archetype: &Archetype) -> bool {
        !archetype.has_component(TypeId::of::<T>())
    }
}
