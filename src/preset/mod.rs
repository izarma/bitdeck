/// Standard-card presets built with [`crate::deck!`].
#[cfg(feature = "cards")]
pub mod cards;

#[cfg(feature = "cards")]
pub use cards::{Aces, Card, Color, FaceCards, Jokers, Rank, Standard, StandardCards, Suit};

#[cfg(feature = "cards")]
pub use cards::{CARD_COUNT, FULL_DECK, JOKER_START};

#[cfg(all(feature = "cards", feature = "alloc"))]
pub use cards::card_name;
