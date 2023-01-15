use crate::{collections::ComponentVec, traits::AsAny};

mod bundle;
mod changes;

pub use bundle::*;
pub(crate) use changes::*;

pub trait Component: AsAny {
    fn get_storage(&self) -> Box<dyn ComponentVec>;
}
