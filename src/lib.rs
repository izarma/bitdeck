#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use rand::{Rng, RngExt};

/// Number of cards in a standard deck (52 playing cards + 2 jokers).
pub const CARD_COUNT: u8 = 54;
/// Bitmask with all [`CARD_COUNT`] bits set; the initial state of a full deck.
pub const FULL_DECK: u64 = (1 << CARD_COUNT) - 1;

/// A standard 52-card deck plus two jokers.
///
/// Cards are identified by number; see the [crate-level documentation](crate)
/// for the mapping and a quick-start example.
///
/// Internally, the set of remaining cards is stored as a 64-bit bitmask, so a
/// deck is `Copy`, allocation-free, and fits in 8 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Resource))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
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
    #[must_use]
    pub fn empty() -> Self {
        Self { mask: 0 }
    }

    /// Wraps a raw bitmask into a deck.
    ///
    /// Bits above [`CARD_COUNT`] are always masked off; in debug builds their
    /// presence also panics, since they are not cards. A mask produced by
    /// [`as_bits`](Self::as_bits) always round-trips.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::StdDeck;
    ///
    /// let deck = StdDeck::default();
    /// assert_eq!(StdDeck::from_bits(deck.as_bits()), deck);
    /// ```
    #[inline]
    #[must_use]
    pub fn from_bits(mask: u64) -> Self {
        debug_assert!(
            mask & !FULL_DECK == 0,
            "bits at or above CARD_COUNT are not cards: {mask:#066b}"
        );
        Self {
            mask: mask & FULL_DECK,
        }
    }

    /// Returns the raw bitmask of the cards that remain.
    ///
    /// The inverse of [`from_bits`](Self::from_bits); handy for persisting a
    /// deck without the `serde` feature or for sending it over the wire.
    #[inline]
    pub fn as_bits(&self) -> u64 {
        self.mask
    }

    /// Restores the deck to a full, unshuffled state.
    #[inline]
    pub fn restock(&mut self) {
        self.mask = FULL_DECK;
    }

    /// Returns the number of cards left in the deck.
    #[inline]
    #[must_use]
    pub fn remaining(&self) -> u32 {
        self.mask.count_ones()
    }

    /// Returns `true` if no cards remain.
    #[inline]
    #[must_use]
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
    /// // `draw` returns `None` once the deck is empty, so it can drive a deal loop.
    /// while let Some(card) = deck.draw(&mut rng) {
    ///     // Give the card to a player.
    ///     let _ = card;
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
    /// use bitdeck::StdDeck;
    /// use rand::{SeedableRng, rngs::SmallRng};
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

    /// Iterates over the cards remaining in the deck, in ascending card-id
    /// order.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::StdDeck;
    ///
    /// let deck = StdDeck::empty();
    /// assert_eq!(deck.iter().count(), 0);
    /// assert_eq!(StdDeck::default().iter().count(), 54);
    /// ```
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter {
        Iter { mask: self.mask }
    }

    /// Returns `true` if the given card is still in the deck.
    ///
    /// # Panics
    ///
    /// Panics if `card >= CARD_COUNT`.
    #[inline]
    #[must_use]
    pub fn contains(&self, card: u8) -> bool {
        assert!(card < CARD_COUNT, "card id {card} out of range");
        (self.mask >> card) & 1 == 1
    }

    /// Puts `card` back into the deck. No effect if it is already present.
    ///
    /// # Panics
    ///
    /// Panics if `card >= CARD_COUNT`.
    #[inline]
    pub fn insert(&mut self, card: u8) {
        assert!(card < CARD_COUNT, "card id {card} out of range");
        self.mask |= 1u64 << card;
    }

    /// Removes `card` from the deck and returns whether it was present.
    ///
    /// # Panics
    ///
    /// Panics if `card >= CARD_COUNT`.
    #[inline]
    pub fn remove(&mut self, card: u8) -> bool {
        let had = self.contains(card);
        self.mask &= !(1u64 << card);
        had
    }
}

/// Iterator over the cards remaining in a [`StdDeck`], in ascending card-id
/// order. Created by [`StdDeck::iter`].
#[derive(Clone, Debug)]
pub struct Iter {
    mask: u64,
}

impl Iterator for Iter {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.mask == 0 {
            return None;
        }
        let bit = self.mask & self.mask.wrapping_neg();
        self.mask &= self.mask - 1;
        Some(bit.trailing_zeros() as u8)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.mask.count_ones() as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for Iter {
    #[inline]
    fn next_back(&mut self) -> Option<u8> {
        if self.mask == 0 {
            return None;
        }
        let idx = self.mask.ilog2();
        self.mask &= !(1u64 << idx);
        Some(idx as u8)
    }
}

impl ExactSizeIterator for Iter {}
impl core::iter::FusedIterator for Iter {}

impl IntoIterator for &StdDeck {
    type Item = u8;
    type IntoIter = Iter;

    #[inline]
    fn into_iter(self) -> Iter {
        self.iter()
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
    fn insert_contains_remove_roundtrip() {
        let mut deck = StdDeck::empty();
        assert!(!deck.contains(7));

        deck.insert(7);
        assert!(deck.contains(7));
        assert_eq!(deck.remaining(), 1);

        // Insert is idempotent.
        deck.insert(7);
        assert_eq!(deck.remaining(), 1);

        assert!(deck.remove(7));
        assert!(!deck.contains(7));
        assert!(deck.is_empty());

        // Removing an absent card reports it.
        assert!(!deck.remove(7));
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn contains_panics_on_out_of_range_card() {
        let _ = StdDeck::default().contains(CARD_COUNT);
    }

    #[test]
    fn bits_roundtrip() {
        let mut deck = StdDeck::default();
        let mut rng = test_rng();
        for _ in 0..17 {
            deck.draw(&mut rng);
        }
        assert_eq!(StdDeck::from_bits(deck.as_bits()), deck);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "not cards")]
    fn from_bits_panics_on_bits_above_card_count_in_debug() {
        let _ = StdDeck::from_bits(u64::MAX);
    }

    #[test]
    fn iter_yields_all_cards_in_ascending_order() {
        let cards: Vec<u8> = StdDeck::default().iter().collect();
        assert_eq!(cards.len(), CARD_COUNT as usize);
        assert!(cards.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn iter_skips_drawn_cards() {
        let mut deck = StdDeck::default();
        assert!(deck.remove(0));
        assert!(deck.remove(53));
        let cards: Vec<u8> = deck.iter().collect();
        assert_eq!(cards.len(), CARD_COUNT as usize - 2);
        assert_eq!(cards.first(), Some(&1));
        assert_eq!(cards.last(), Some(&52));
    }

    #[test]
    fn into_iter_on_reference_matches_iter() {
        let deck = StdDeck::default();
        let collected: Vec<u8> = (&deck).into_iter().collect();
        assert_eq!(collected, deck.iter().collect::<Vec<_>>());
    }

    #[test]
    fn iter_len_and_double_ended() {
        let mut iter = StdDeck::default().iter();
        assert_eq!(iter.len(), CARD_COUNT as usize);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(53));
        assert_eq!(iter.len(), CARD_COUNT as usize - 2);
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
