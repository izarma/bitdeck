#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod deck;
mod meaning;

pub use deck::*;
pub use meaning::*;

#[cfg(test)]
mod tests;
