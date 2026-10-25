//! Expression evaluation and the GTK calculator overlay.

#![warn(missing_docs)]

mod app;
mod clipboard_helper;
mod parser;

pub use app::run;
pub use parser::calculate;
