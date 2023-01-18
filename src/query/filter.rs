use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use super::QuerySelector;

pub struct Without<T>(PhantomData<T>);

impl<T: 'static> QuerySelector for Without<T> {
    fn filter(type_ids: &HashSet<TypeId>) -> bool {
        !type_ids.contains(&TypeId::of::<T>())
    }
}
