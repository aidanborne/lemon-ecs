pub mod component;
pub mod engine;
pub mod query;
pub(crate) mod storage;
pub mod system;
pub mod world;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use super::component::prelude::*;
    pub use super::engine::Engine;
    pub use super::query::prelude::*;
    pub use super::storage::prelude::*;
    pub use super::system::prelude::*;
    pub use super::world::prelude::*;

    pub use lemon_ecs_macros::{Bundleable, Component};
}
