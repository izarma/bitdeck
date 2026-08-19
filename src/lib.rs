#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc = include_str!("README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod deck;
mod preset;
mod subset;

pub use deck::*;
pub use subset::*;

#[cfg(feature = "cards")]
pub use preset::*;

#[cfg(feature = "alloc")]
#[doc(hidden)]
pub mod __alloc {
    pub use ::alloc::vec::Vec;
}

#[cfg(feature = "rand")]
#[doc(hidden)]
pub mod __rand {
    pub use ::rand::Rng;
}

#[cfg(test)]
mod tests;
