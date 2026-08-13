//! Standard-card suits, ranks, colors, and predefined masks.
//!
//! Card ids `0..=51` are standard cards (`suit = id / 13`, `rank = id % 13`);
//! ids `52` and `53` are jokers. Jokers are unmapped in every meaning here, so
//! [`Suit::from_id`], [`Rank::from_id`], and [`Color::from_id`] panic for them.
//! Handle them separately, for example with [`JOKERS`].

use crate::{meanings, stride_mask};

meanings! {
    /// The four standard suits (`id / 13`).
    pub enum Suit {
        /// Ids `0..=12`.
        Clubs,
        /// Ids `13..=25`.
        Diamonds,
        /// Ids `26..=38`.
        Hearts,
        /// Ids `39..=51`.
        Spades,
    }
    from_id = |id: u8| id / 13;
    cards = 52;
}

meanings! {
    /// The 13 ranks (`id % 13`).
    pub enum Rank {
        /// Id | start 0 | step 13 | count 4 |
        Ace,
        /// Id | start 1 | step 13 | count 4 |
        Two,
        /// Id | start 2 | step 13 | count 4 |
        Three,
        /// Id | start 3 | step 13 | count 4 |
        Four,
        /// Id | start 4 | step 13 | count 4 |
        Five,
        /// Id | start 5 | step 13 | count 4 |
        Six,
        /// Id | start 6 | step 13 | count 4 |
        Seven,
        /// Id | start 7 | step 13 | count 4 |
        Eight,
        /// Id | start 8 | step 13 | count 4 |
        Nine,
        /// Id | start 9 | step 13 | count 4 |
        Ten,
        /// Id | start 10 | step 13 | count 4 |
        Jack,
        /// Id | start 11 | step 13 | count 4 |
        Queen,
        /// Id | start 12 | step 13 | count 4 |
        King,
    }
    from_id = |id: u8| id % 13;
    cards = 52;
}

meanings! {
    /// Red or black. Jokers are unmapped.
    pub enum Color {
        /// Diamonds and hearts.
        Red,
        /// Clubs and spades.
        Black,
    }
    from_id = |id: u8| match id / 13 { 1 | 2 => 0, _ => 1 };
    cards = 52;
}

/// The two jokers (ids 52 and 53).
pub const JOKERS: u64 = stride_mask(52, 1, 2);
/// Every non-joker card (ids `0..=51`).
pub const STANDARD: u64 = Suit::ALL;
/// Every red card: diamonds and hearts.
pub const RED: u64 = Color::Red.mask();
/// Every black card: clubs and spades.
pub const BLACK: u64 = Color::Black.mask();
/// The four aces.
pub const ACES: u64 = Rank::Ace.mask();
/// Jacks, queens, and kings across all suits.
pub const FACE_CARDS: u64 = Rank::Jack.mask() | Rank::Queen.mask() | Rank::King.mask();
