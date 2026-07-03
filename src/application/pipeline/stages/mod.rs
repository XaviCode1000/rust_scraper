//! Pipeline processing stages.
//!
//! Each stage implements [`PipelineStage`] and performs a single,
//! well-defined transformation or validation on [`ScrapedItem`]s.

mod clean;
mod validate;

pub use clean::CleanStage;
pub use validate::ValidateStage;
