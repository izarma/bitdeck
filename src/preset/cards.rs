//! Standard-card suits, ranks, colors, and predefined subsets.
//!
//! Card ids `0..=51` are standard cards (`suit = id / 13`, `rank = id % 13`);
//! ids `52` and `53` are jokers. Jokers are unmapped in every classification
//! here, so [`crate::Suit::from_id`], [`crate::Rank::from_id`],
//! and [`crate::Color::from_id`] panic for them. Handle them separately,
//! for example with [`crate::Jokers`] or [`crate::Card`].

use crate::deck;

/// Number of cards in a [`Standard`] deck.
pub const CARD_COUNT: u8 = 54;
/// Bitmask with all [`CARD_COUNT`] bits set; the initial state of a full [`Standard`] deck.
pub const FULL_DECK: u64 = crate::full_mask::<CARD_COUNT>();
/// First card id used by a joker.
pub const JOKER_START: u8 = 52;

deck! {
    /// A standard 52-card deck plus two jokers.
    pub struct Standard = Deck<54>;

    subsets {
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

        /// The 13 ranks (`id % 13`).
        pub enum Rank {
            /// Ace: ids 0, 13, 26, 39.
            Ace,
            /// 2: ids 1, 14, 27, 40.
            Two,
            /// 3: ids 2, 15, 28, 41.
            Three,
            /// 4: ids 3, 16, 29, 42.
            Four,
            /// 5: ids 4, 17, 30, 43.
            Five,
            /// 6: ids 5, 18, 31, 44.
            Six,
            /// 7: ids 6, 19, 32, 45.
            Seven,
            /// 8: ids 7, 20, 33, 46.
            Eight,
            /// 9: ids 8, 21, 34, 47.
            Nine,
            /// 10: ids 9, 22, 35, 48.
            Ten,
            /// 11: ids 10, 23, 36, 49.
            Jack,
            /// 12: ids 11, 24, 37, 50.
            Queen,
            /// 13: ids 12, 25, 38, 51.
            King,
        }
        from_id = |id: u8| id % 13;
        cards = 52;

        /// Red or black. Jokers are unmapped.
        pub enum Color {
            /// Diamonds and hearts.
            Red,
            /// Clubs and spades.
            Black,
        }
        from_id = |id: u8| match id / 13 { 1 | 2 => 0, _ => 1 };
        cards = 52;

        /// The two jokers (ids [`JOKER_START`] and [`JOKER_START`] + 1).
        pub struct Jokers {
            mask = (1u64 << JOKER_START) | (1u64 << (JOKER_START + 1));
        }

        /// Every non-joker card (ids `0..=51`).
        pub struct StandardCards {
            mask = Suit::ALL;
        }

        /// The four aces.
        pub struct Aces {
            mask = Rank::Ace.mask();
        }

        /// Jacks, queens, and kings across all suits.
        pub struct FaceCards {
            mask = Rank::Jack.mask() | Rank::Queen.mask() | Rank::King.mask();
        }
    }
}

impl Suit {
    /// Unicode symbol for the suit.
    #[inline]
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }
}

/// A standard playing card or joker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Card {
    /// A standard card with a suit and rank.
    Standard(Suit, Rank),
    /// One of the two jokers (`0` or `1`).
    Joker(u8),
}

impl Card {
    /// Builds a [`Card`] from a [`Standard`] card id.
    ///
    /// # Panics
    ///
    /// Panics if `id >= CARD_COUNT`.
    #[inline]
    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        assert!(id < CARD_COUNT, "card id out of range");
        if id >= JOKER_START {
            Card::Joker(id - JOKER_START)
        } else {
            Card::Standard(Suit::from_id(id), Rank::from_id(id))
        }
    }
}

impl core::fmt::Display for Card {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Card::Standard(suit, rank) => write!(f, "{:#?}{}", rank, suit.symbol()),
            Card::Joker(n) => write!(f, "Joker {n}"),
        }
    }
}

/// Returns a human-readable name for a [`Standard`] card id.
///
/// # Panics
///
/// Panics if `id >= CARD_COUNT`.
#[cfg(feature = "alloc")]
#[inline]
#[must_use]
pub fn card_name(id: u8) -> alloc::string::String {
    alloc::format!("{}", Card::from_id(id))
}
