#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod deck;
mod meaning;

pub use deck::*;
pub use meaning::*;

#[cfg(test)]
mod tests;
