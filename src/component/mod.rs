use crate::downcast::{impl_downcast, AsAny};

mod bundle;
mod vec;

pub use bundle::*;
pub use vec::*;

pub trait Component: AsAny {
    fn as_empty_vec(&self) -> Box<dyn ComponentVec>;
}

impl_downcast!(dyn Component);
