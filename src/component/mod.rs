use crate::{
    collections::ComponentVec,
    downcast::{impl_downcast, AsAny},
};

mod bundle;

pub use bundle::*;

pub trait Component: AsAny {
    fn as_empty_vec(&self) -> Box<dyn ComponentVec>;
}

impl_downcast!(dyn Component);
