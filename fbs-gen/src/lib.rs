//! Random FlatBuffers schema generator.
//!
//! Produces valid `.fbs` text that is guaranteed to parse, analyze, and codegen
//! successfully. Uses a deterministic seed so the same seed always produces the
//! same schema -- failing seeds are trivially reproducible.
//!
//! Every random decision and probability is configurable via [`GenConfig`].

mod builder;
pub mod chooser;
mod config;
mod defaults;
mod gen_enum;
mod gen_meta;
mod gen_struct;
mod gen_table;
mod gen_union;
mod names;
mod types;

pub use builder::SchemaBuilder;
pub use chooser::{Chooser, RngChooser, ScriptedChooser};
pub use config::GenConfig;

#[cfg(test)]
mod tests;
