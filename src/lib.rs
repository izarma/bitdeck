#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod deck;
pub use deck::{CARD_COUNT, FULL_DECK, StdDeck};
