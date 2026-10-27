//! Bounded native translation of the Prettier 2.5.1 GraphQL printer.
mod document;
mod format;
pub use format::{format, FormatError, Indentation, MAX_INPUT, MAX_OUTPUT};
