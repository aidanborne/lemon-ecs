use crate::{collections::ComponentVec, traits::AsAny};

mod bundle;

pub use bundle::*;

pub trait Component: AsAny {
    fn get_storage(&self) -> Box<dyn ComponentVec>;
}
