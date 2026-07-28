//! alfredrs — Alfred-inspired keyboard launcher for Linux.
//!
//! Inspired by [Alfred](https://www.alfredapp.com). Not affiliated with
//! Running with Crayons Ltd.

pub mod config;
pub mod engine;
pub mod hotkey;
pub mod model;
pub mod paths;
pub mod providers;
pub mod ranking;
pub mod ui;

pub use config::Config;
pub use engine::Engine;
pub use model::{Action, ItemKind, Query, ResultItem};
