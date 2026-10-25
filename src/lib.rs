//! Expression evaluation and the GTK calculator overlay.

#![warn(missing_docs)]

mod angle_mode;
mod app;
mod clipboard_helper;
mod parser;

pub use angle_mode::AngleMode;
pub use app::run;
pub use parser::{calculate, calculate_with_angle_mode};
