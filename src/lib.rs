#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use rand::{Rng, RngExt};

/// Number of cards in a standard deck (52 playing cards + 2 jokers).
pub const CARD_COUNT: u8 = 54;
/// Bitmask with all [`CARD_COUNT`] bits set; the initial state of a full deck.
pub const FULL_DECK: u64 = (1 << CARD_COUNT) - 1;

/// A standard 52-card deck plus two jokers, represented as a bitmask of the
/// cards that remain.
///
/// Card identity is just a number:
///   - `0..=51` -> standard cards (`suit = id / 13`, `rank = id % 13`)
///   - `52, 53` -> the two jokers
///
/// Deck state is which cards remain, stored as one 64-bit bitmask.
///
/// See the [crate-level documentation](crate) for a quick-start example.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Resource))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StdDeck {
    mask: u64,
}

impl Default for StdDeck {
    fn default() -> Self {
        Self { mask: FULL_DECK }
    }
}

impl StdDeck {
    /// Returns a deck with no cards remaining.
    ///
    /// Drawing from it returns `None` until cards are added back with
    /// [`insert`](Self::insert) or [`restock`](Self::restock).
    #[inline]
    pub fn empty() -> Self {
        Self { mask: 0 }
    }

    /// Restores the deck to a full, unshuffled state.
    #[inline]
    pub fn restock(&mut self) {
        self.mask = FULL_DECK;
    }

    /// Returns the number of cards left in the deck.
    #[inline]
    pub fn remaining(&self) -> u32 {
        self.mask.count_ones()
    }

    /// Returns `true` if no cards remain.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    /// Draws one random card from the deck, removing it.
    ///
    /// Returns `None` if the deck is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::StdDeck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let mut deck = StdDeck::default();
    /// let mut rng = SmallRng::seed_from_u64(7);
    ///
    /// if let Some(card) = deck.draw(&mut rng) {
    ///     println!("drew card {card}");
    /// }
    /// ```
    #[inline]
    pub fn draw(&mut self, rng: &mut impl Rng) -> Option<u8> {
        if self.mask == 0 {
            return None;
        }
        let k = rng.random_range(0..self.remaining());
        let bit = select_nth_set(self.mask, k);
        self.mask &= !bit;
        Some(bit.trailing_zeros() as u8)
    }

    /// Draw up to `count` cards into `out`.
    ///
    /// The buffer is cleared first. If the deck runs out before `count` cards
    /// are drawn, the function stops early and returns the number of cards
    /// actually drawn, so the caller can decide whether to reshuffle, error,
    /// or proceed with a partial hand.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitdeck::StdDeck;
    /// # use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let mut deck = StdDeck::default();
    /// let mut rng = SmallRng::seed_from_u64(99);
    /// let mut hand = Vec::new();
    ///
    /// // Ask for 5 cards and check the count to detect a short deck.
    /// let drawn = deck.draw_into(&mut rng, 5, &mut hand);
    /// if drawn < 5 {
    ///     // Reshuffle, error, or play the partial hand.
    /// }
    /// ```
    pub fn draw_into(&mut self, rng: &mut impl Rng, count: usize, out: &mut Vec<u8>) -> usize {
        out.clear();
        out.reserve(count.min(self.remaining() as usize));
        for _ in 0..count {
            match self.draw(rng) {
                Some(c) => out.push(c),
                None => break,
            }
        }
        out.len()
    }

    /// Returns `true` if the given card is still in the deck.
    #[inline]
    pub fn contains(&self, card: u8) -> bool {
        debug_assert!(card < CARD_COUNT, "card id {card} out of range");
        (self.mask >> card) & 1 == 1
    }

    /// Puts `card` back into the deck. No effect if it is already present.
    #[inline]
    pub fn insert(&mut self, card: u8) {
        debug_assert!(card < CARD_COUNT, "card id {card} out of range");
        self.mask |= 1u64 << card;
    }

    /// Removes `card` from the deck and returns whether it was present.
    #[inline]
    pub fn remove(&mut self, card: u8) -> bool {
        debug_assert!(card < CARD_COUNT, "card id {card} out of range");
        let bit = 1u64 << card;
        let had = (self.mask & bit) != 0;
        self.mask &= !bit;
        had
    }
}

#[inline]
fn select_nth_set(mask: u64, k: u32) -> u64 {
    debug_assert!(
        k < mask.count_ones(),
        "k ({k}) must be less than the popcount of the mask ({})",
        mask.count_ones()
    );
    #[cfg(target_arch = "x86_64")]
    {
        // Explore later how much impact this check actually has due to branch prediction.
        if std::arch::is_x86_feature_detected!("bmi2") {
            // SAFETY: BMI2 confirmed at runtime.
            return unsafe { core::arch::x86_64::_pdep_u64(1u64 << k, mask) };
        }
    }
    // Portable fallback: clear the lowest set bit k times, then isolate it.
    let mut m = mask;
    for _ in 0..k {
        m &= m - 1; // Brian Kernighan's algorithm
    }
    m & m.wrapping_neg()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::SmallRng};

    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(0x1234_5678_9ABC_DEF0)
    }

    #[test]
    fn draw_all_cards_leaves_deck_empty() {
        let mut deck = StdDeck::default();
        let mut rng = test_rng();
        let mut drawn = Vec::with_capacity(CARD_COUNT as usize);
        while let Some(card) = deck.draw(&mut rng) {
            drawn.push(card);
        }
        assert_eq!(drawn.len(), CARD_COUNT as usize);
        assert!(deck.is_empty());
        // Every card drawn exactly once.
        drawn.sort_unstable();
        drawn.dedup();
        assert_eq!(drawn.len(), CARD_COUNT as usize);
    }

    #[test]
    fn draw_is_deterministic_with_seed() {
        let mut a = StdDeck::default();
        let mut b = StdDeck::default();
        for _ in 0..10 {
            assert_eq!(a.draw(&mut test_rng()), b.draw(&mut test_rng()));
        }
    }

    #[test]
    fn draw_into_respects_deck_size() {
        let mut deck = StdDeck::default();
        let mut out = Vec::new();
        let drawn = deck.draw_into(&mut test_rng(), 100, &mut out);
        assert_eq!(drawn, CARD_COUNT as usize);
        assert_eq!(out.len(), CARD_COUNT as usize);
        assert!(deck.is_empty());
    }

    #[test]
    fn select_nth_set_first_and_last() {
        let mask = 0b1010_1100u64;
        assert_eq!(select_nth_set(mask, 0), 0b0000_0100);
        assert_eq!(select_nth_set(mask, 1), 0b0000_1000);
        assert_eq!(select_nth_set(mask, 2), 0b0010_0000);
        assert_eq!(select_nth_set(mask, 3), 0b1000_0000);
    }

    #[test]
    fn select_nth_set_matches_popcount_positions() {
        let mask = FULL_DECK;
        for k in [0, 1, 13, 27, 52, 53] {
            let bit = select_nth_set(mask, k);
            assert_eq!(bit.count_ones(), 1);
            assert_eq!(bit.trailing_zeros(), k);
        }
    }
}
