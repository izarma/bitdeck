#[cfg(feature = "rand")]
use rand::{Rng, RngExt};

#[cfg(all(feature = "rand", feature = "alloc"))]
use alloc::vec::Vec;

#[cfg(feature = "bevy")]
use bevy_ecs::prelude::ReflectComponent;

/// Bitmask with the lowest `N` bits set; every card of a [`Deck<N>`].
///
/// # Panics
///
/// Panics if `N > 64`.
#[must_use]
pub const fn full_mask<const N: u8>() -> u64 {
    assert!(N <= 64, "deck size must fit in a u64 bitmask");
    if N == 64 { u64::MAX } else { (1u64 << N) - 1 }
}

/// A deck of `N` cards represented as a bitmask of the cards that remain.
///
/// Card identity is just a number `0..N`. Deck state is stored as one 64-bit
/// bitmask, so `N` must be `<= 64`.
///
/// The `cards` feature provides a standard 52-card deck plus two jokers.
///
/// Subset APIs accept a `u64` mask of card ids, typically built with
/// [`crate::deck!`]. Bits at or above `N` are ignored by subset queries,
/// mutations, and draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "bevy",
    derive(bevy_ecs::prelude::Component, bevy_reflect::Reflect)
)]
#[cfg_attr(feature = "bevy", reflect(Component))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Deck<const N: u8> {
    mask: u64,
}

impl<const N: u8> Default for Deck<N> {
    fn default() -> Self {
        const { assert!(N <= 64, "deck size must fit in a u64 bitmask") };
        Self {
            mask: full_mask::<N>(),
        }
    }
}

impl<const N: u8> Deck<N> {
    /// Returns a deck with no cards remaining.
    ///
    /// Drawing from it returns `None` until cards are added back with
    /// [`insert`](Self::insert) or [`restock`](Self::restock).
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        const { assert!(N <= 64, "deck size must fit in a u64 bitmask") };
        Self { mask: 0 }
    }

    /// Wraps a raw bitmask into a deck.
    ///
    /// Bits at or above `N` are always masked off; in debug builds their
    /// presence also panics, since they are not cards. A mask produced by
    /// [`as_bits`](Self::as_bits) always round-trips.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    ///
    /// let deck = Deck::<54>::default();
    /// assert_eq!(Deck::<54>::from_bits(deck.as_bits()), deck);
    /// ```
    #[inline]
    #[must_use]
    pub fn from_bits(mask: u64) -> Self {
        const { assert!(N <= 64, "deck size must fit in a u64 bitmask") };
        debug_assert!(
            mask & !full_mask::<N>() == 0,
            "bits at or above deck size are not cards: {mask:#066b}"
        );
        Self {
            mask: mask & full_mask::<N>(),
        }
    }

    /// Returns the raw bitmask of the cards that remain.
    ///
    /// The inverse of [`from_bits`](Self::from_bits); handy for persisting a
    /// deck without the `serde` feature or for sending it over the wire.
    #[inline]
    #[must_use]
    pub const fn as_bits(&self) -> u64 {
        self.mask
    }

    /// Restores the deck to a full state.
    #[inline]
    pub const fn restock(&mut self) {
        self.mask = full_mask::<N>();
    }

    /// Puts every card selected by `mask` back into the deck.
    #[inline]
    pub const fn insert_all(&mut self, mask: u64) {
        self.mask |= mask & full_mask::<N>();
    }

    /// Removes every remaining card selected by `mask` and returns how many
    /// were removed.
    #[inline]
    pub const fn remove_all(&mut self, mask: u64) -> u32 {
        let removed = self.count_in(mask);
        self.mask &= !mask;
        removed
    }

    /// Keeps only the remaining cards selected by `mask`.
    #[inline]
    pub const fn retain(&mut self, mask: u64) {
        self.mask &= mask;
    }

    /// Returns the number of cards left in the deck.
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> u32 {
        self.mask.count_ones()
    }

    /// Returns `true` if no cards remain.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.mask == 0
    }

    /// Returns `true` if every card remains in the deck.
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.mask == full_mask::<N>()
    }

    /// Returns the raw bitmask of cards that have been drawn.
    #[inline]
    #[must_use]
    pub const fn drawn_mask(&self) -> u64 {
        !self.mask & full_mask::<N>()
    }

    /// Returns the number of cards that have been drawn.
    #[inline]
    #[must_use]
    pub const fn drawn_count(&self) -> u32 {
        N as u32 - self.remaining()
    }

    /// Iterates over drawn cards in ascending card-id order.
    #[inline]
    #[must_use]
    pub const fn iter_drawn(&self) -> Iter {
        Iter {
            mask: self.drawn_mask(),
        }
    }

    /// Iterates over remaining cards selected by `mask`, in ascending card-id
    /// order.
    #[inline]
    #[must_use]
    pub const fn iter_in(&self, mask: u64) -> Iter {
        Iter {
            mask: self.mask & mask,
        }
    }

    /// Iterates over the cards remaining in the deck, in ascending card-id
    /// order.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    ///
    /// assert_eq!(Deck::<54>::empty().iter().count(), 0);
    /// assert_eq!(Deck::<54>::default().iter().count(), 54);
    /// ```
    #[inline]
    #[must_use]
    pub const fn iter(&self) -> Iter {
        Iter { mask: self.mask }
    }

    /// Returns the `k`th remaining card in ascending card-id order.
    ///
    /// Returns `None` when `k` is greater than or equal to
    /// [`remaining`](Self::remaining).
    #[inline]
    #[must_use]
    pub fn nth(&self, k: u32) -> Option<u8> {
        (k < self.remaining()).then(|| card_id(select_nth_set(self.mask, k)))
    }

    /// Returns the lowest-id remaining card.
    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<u8> {
        (!self.is_empty()).then(|| card_id(self.mask.isolate_lowest_one()))
    }

    /// Returns the highest-id remaining card.
    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<u8> {
        (!self.is_empty()).then(|| self.mask.ilog2() as u8)
    }

    /// Returns the `k`th remaining card selected by `mask`, in ascending
    /// card-id order.
    ///
    /// Returns `None` when `k` is greater than or equal to
    /// [`count_in(mask)`](Self::count_in).
    #[inline]
    #[must_use]
    pub fn nth_in(&self, mask: u64, k: u32) -> Option<u8> {
        let subset = self.mask & mask;
        (k < subset.count_ones()).then(|| card_id(select_nth_set(subset, k)))
    }

    /// Returns the lowest-id remaining card selected by `mask`.
    #[inline]
    #[must_use]
    pub fn first_in(&self, mask: u64) -> Option<u8> {
        let subset = self.mask & mask;
        (subset != 0).then(|| card_id(select_nth_set(subset, 0)))
    }

    /// Returns the highest-id remaining card selected by `mask`.
    #[inline]
    #[must_use]
    pub fn last_in(&self, mask: u64) -> Option<u8> {
        let subset = self.mask & mask;
        (subset != 0).then(|| card_id(select_nth_set(subset, subset.count_ones() - 1)))
    }

    /// Randomly selects a remaining card without removing it.
    ///
    /// Returns `None` if the deck is empty.
    #[cfg(feature = "rand")]
    #[inline]
    pub fn peek(&self, rng: &mut impl Rng) -> Option<u8> {
        self.peek_in(rng, full_mask::<N>())
    }

    /// Randomly selects a remaining card from `mask` without removing it.
    ///
    /// Returns `None` if no selected card remains.
    #[cfg(feature = "rand")]
    #[inline]
    pub fn peek_in(&self, rng: &mut impl Rng, mask: u64) -> Option<u8> {
        let subset = self.mask & mask;
        if subset == 0 {
            return None;
        }
        let k = rng.random_range(0..subset.count_ones());
        Some(card_id(select_nth_set(subset, k)))
    }

    /// Peeks up to `count` random cards from the deck and returns them as a
    /// bitmask without removing them.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let deck = Deck::<16>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// let seen = deck.peek_mask(&mut rng, 5);
    /// assert_eq!(seen.count_ones(), 5);
    /// assert_eq!(deck.remaining(), 16);
    /// ```
    #[cfg(feature = "rand")]
    pub fn peek_mask(&self, rng: &mut impl Rng, count: usize) -> u64 {
        self.peek_in_mask(rng, full_mask::<N>(), count)
    }

    /// Peeks up to `count` random cards from a subset of the deck and returns
    /// them as a bitmask without removing them.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// const LOW: u64 = 0b1111; // ids 0, 1, 2, 3
    ///
    /// let deck = Deck::<16>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// let seen = deck.peek_in_mask(&mut rng, LOW, 10);
    /// assert_eq!(seen.count_ones(), 4);
    /// assert!(seen & !LOW == 0);
    /// assert_eq!(deck.count_in(LOW), 4);
    /// ```
    #[cfg(feature = "rand")]
    pub fn peek_in_mask(&self, rng: &mut impl Rng, mask: u64, count: usize) -> u64 {
        let mut peeked = 0u64;
        let mut subset = self.mask & mask;
        let n = count.min(subset.count_ones() as usize);
        for _ in 0..n {
            let k = rng.random_range(0..subset.count_ones());
            let bit = select_nth_set(subset, k);
            peeked |= bit;
            subset &= !bit;
        }
        peeked
    }

    /// Draws one random card from the deck, removing it.
    ///
    /// Returns `None` if the deck is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let mut deck = Deck::<54>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// // `draw` returns `None` once the deck is empty, so it can drive a deal loop.
    /// while let Some(card) = deck.draw(&mut rng) {
    ///     // Give the card to a player.
    ///     let _ = card;
    /// }
    /// ```
    #[cfg(feature = "rand")]
    #[inline]
    pub fn draw(&mut self, rng: &mut impl Rng) -> Option<u8> {
        self.draw_in(rng, full_mask::<N>())
    }

    /// Draws one random card from a subset of the deck, removing it.
    ///
    /// Returns `None` if no card of the subset remains.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// // A subset is a mask with one bit per card ID.
    /// const SUBSET: u64 = (1 << 1) | (1 << 4) | (1 << 6);
    ///
    /// let mut deck = Deck::<8>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// let card = deck.draw_in(&mut rng, SUBSET).unwrap();
    /// assert_ne!(SUBSET & (1 << card), 0);
    /// assert_eq!(deck.count_in(SUBSET), 2);
    /// ```
    #[cfg(feature = "rand")]
    #[inline]
    pub fn draw_in(&mut self, rng: &mut impl Rng, mask: u64) -> Option<u8> {
        let subset = self.mask & mask;
        if subset == 0 {
            return None;
        }
        let k = rng.random_range(0..subset.count_ones());
        let bit = select_nth_set(subset, k);
        self.mask &= !bit;
        Some(card_id(bit))
    }

    /// Draws up to `count` random cards from the deck and returns them as a
    /// bitmask.
    ///
    /// The returned bitmask has one bit set for every card that was removed.
    /// Use [`count_ones`](u64::count_ones) or iterate with
    /// [`Deck::from_bits`](Self::from_bits) to inspect the result.
    ///
    /// If fewer than `count` cards remain, all remaining cards are drawn.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let mut deck = Deck::<16>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// let hand = deck.draw_mask(&mut rng, 5);
    /// assert_eq!(hand.count_ones(), 5);
    /// assert_eq!(deck.remaining(), 11);
    /// ```
    #[cfg(feature = "rand")]
    pub fn draw_mask(&mut self, rng: &mut impl Rng, count: usize) -> u64 {
        self.draw_in_mask(rng, full_mask::<N>(), count)
    }

    /// Draws up to `count` random cards from a subset of the deck and returns
    /// them as a bitmask.
    ///
    /// If fewer than `count` selected cards remain, all of them are drawn.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// const LOW: u64 = 0b1111; // ids 0, 1, 2, 3
    ///
    /// let mut deck = Deck::<16>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// let hand = deck.draw_in_mask(&mut rng, LOW, 10);
    /// assert_eq!(hand.count_ones(), 4);
    /// assert!(hand & !LOW == 0);
    /// assert_eq!(deck.count_in(LOW), 0);
    /// ```
    #[cfg(feature = "rand")]
    pub fn draw_in_mask(&mut self, rng: &mut impl Rng, mask: u64, count: usize) -> u64 {
        let mut drawn = 0u64;
        let mut subset = self.mask & mask;
        let n = count.min(subset.count_ones() as usize);
        for _ in 0..n {
            let k = rng.random_range(0..subset.count_ones());
            let bit = select_nth_set(subset, k);
            drawn |= bit;
            subset &= !bit;
        }
        self.mask &= !drawn;
        drawn
    }

    /// Draws up to `count` cards into `out`.
    ///
    /// The buffer is cleared first. If the deck runs out before `count` cards
    /// are drawn, the function stops early and returns the number of cards
    /// actually drawn, so the caller can decide whether to reshuffle, error,
    /// or proceed with a partial hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let mut deck = Deck::<54>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    /// let mut hand = Vec::new();
    ///
    /// // Ask for 5 cards and check the count to detect a short deck.
    /// let drawn = deck.draw_into(&mut rng, 5, &mut hand);
    /// if drawn < 5 {
    ///     // Reshuffle, error, or play the partial hand.
    /// }
    /// ```
    #[cfg(all(feature = "rand", feature = "alloc"))]
    pub fn draw_into(&mut self, rng: &mut impl Rng, count: usize, out: &mut Vec<u8>) -> usize {
        let mask = self.draw_mask(rng, count);
        out.clear();
        out.extend(Iter { mask });
        mask.count_ones() as usize
    }

    /// Draws up to `count` cards from a subset of the deck into `out`.
    ///
    /// The buffer is cleared first. If the subset runs out before `count` cards
    /// are drawn, the function stops early and returns the number of cards
    /// actually drawn.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// const HEARTS: u64 = ((1u64 << 13) - 1) << 26; // ids 26..=38
    ///
    /// let mut deck = Deck::<54>::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    /// let mut hand = Vec::new();
    ///
    /// let drawn = deck.draw_in_into(&mut rng, HEARTS, 5, &mut hand);
    /// assert_eq!(drawn, 5);
    /// for card in &hand {
    ///     assert_eq!(card / 13, 2); // every drawn card is a heart
    /// }
    /// ```
    #[cfg(all(feature = "rand", feature = "alloc"))]
    pub fn draw_in_into(
        &mut self,
        rng: &mut impl Rng,
        mask: u64,
        count: usize,
        out: &mut Vec<u8>,
    ) -> usize {
        let drawn = self.draw_in_mask(rng, mask, count);
        out.clear();
        out.extend(Iter { mask: drawn });
        drawn.count_ones() as usize
    }

    /// Peeks up to `count` random cards into `out` without removing them.
    ///
    /// The buffer is cleared first. If fewer than `count` cards remain, all of
    /// them are peeked. The deck is unchanged.
    #[cfg(all(feature = "rand", feature = "alloc"))]
    pub fn peek_into(&self, rng: &mut impl Rng, count: usize, out: &mut Vec<u8>) -> usize {
        let mask = self.peek_mask(rng, count);
        out.clear();
        out.extend(Iter { mask });
        mask.count_ones() as usize
    }

    /// Peeks up to `count` random cards from a subset of the deck into `out`,
    /// without removing them.
    ///
    /// The buffer is cleared first. If the subset contains fewer than `count`
    /// cards, all of them are peeked. The deck is unchanged.
    #[cfg(all(feature = "rand", feature = "alloc"))]
    pub fn peek_in_into(
        &self,
        rng: &mut impl Rng,
        mask: u64,
        count: usize,
        out: &mut Vec<u8>,
    ) -> usize {
        let peeked = self.peek_in_mask(rng, mask, count);
        out.clear();
        out.extend(Iter { mask: peeked });
        peeked.count_ones() as usize
    }

    /// Returns `true` if the given card is still in the deck.
    ///
    /// # Panics
    ///
    /// Panics if `card >= N`.
    #[inline]
    #[must_use]
    pub fn contains(&self, card: u8) -> bool {
        assert!(card < N, "card id {card} out of range");
        (self.mask >> card) & 1 == 1
    }

    /// Returns `true` if any card of the subset `mask` is still in the deck.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    ///
    /// // Each set bit selects a card ID.
    /// const SUBSET: u64 = (1 << 1) | (1 << 4) | (1 << 6);
    ///
    /// let deck = Deck::<8>::default();
    /// assert!(deck.contains_any(SUBSET));
    /// ```
    #[inline]
    #[must_use]
    pub const fn contains_any(&self, mask: u64) -> bool {
        self.mask & mask != 0
    }

    /// Returns `true` if every card of the subset `mask` is still in the deck.
    ///
    /// Bits at or above `N` are ignored.
    #[inline]
    #[must_use]
    pub const fn contains_all(&self, mask: u64) -> bool {
        let mask = mask & full_mask::<N>();
        self.mask & mask == mask
    }

    /// Returns `true` if the remaining cards are a subset of `mask`.
    ///
    /// An empty deck is a subset of every mask (vacuously true), matching
    /// [`contains_all`](Self::contains_all) on an empty deck.
    #[inline]
    #[must_use]
    pub const fn is_subset_of(&self, mask: u64) -> bool {
        self.mask & !mask == 0
    }

    /// Returns how many cards of the subset `mask` are still in the deck.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    ///
    /// const HEARTS: u64 = ((1u64 << 13) - 1) << 26; // ids 26..=38
    ///
    /// let deck = Deck::<54>::default();
    /// assert_eq!(deck.count_in(HEARTS), 13);
    /// ```
    #[inline]
    #[must_use]
    pub const fn count_in(&self, mask: u64) -> u32 {
        (self.mask & mask).count_ones()
    }

    /// Returns the probability that a uniformly random remaining card is in
    /// `mask`.
    ///
    /// Returns `NaN` when the deck is empty.
    #[inline]
    #[must_use]
    pub fn chance(&self, mask: u64) -> f64 {
        self.count_in(mask) as f64 / self.remaining() as f64
    }

    /// Puts `card` back into the deck. No effect if it is already present.
    ///
    /// # Panics
    ///
    /// Panics if `card >= N`.
    #[inline]
    pub fn insert(&mut self, card: u8) {
        assert!(card < N, "card id {card} out of range");
        self.mask |= 1u64 << card;
    }

    /// Removes `card` from the deck and returns whether it was present.
    ///
    /// # Panics
    ///
    /// Panics if `card >= N`.
    #[inline]
    pub fn remove(&mut self, card: u8) -> bool {
        let had = self.contains(card);
        self.mask &= !(1u64 << card);
        had
    }

    /// Flips whether `card` is in the deck and returns its new presence.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::Deck;
    ///
    /// let mut deck = Deck::<8>::empty();
    /// assert!(deck.toggle(3));
    /// assert!(deck.contains(3));
    /// assert!(!deck.toggle(3));
    /// assert!(!deck.contains(3));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `card >= N`.
    #[inline]
    pub fn toggle(&mut self, card: u8) -> bool {
        assert!(card < N, "card id {card} out of range");
        self.mask ^= 1u64 << card;
        (self.mask >> card) & 1 == 1
    }

    /// Flips the presence of every card selected by `mask`.
    ///
    /// Bits at or above `N` are masked off; toggling `u64::MAX` on a deck with
    /// `N < 64` only flips the in-range bits.
    #[inline]
    pub const fn toggle_all(&mut self, mask: u64) {
        self.mask ^= mask & full_mask::<N>();
    }
}

/// Iterator over selected card ids in ascending card-id order. Created by
/// [`Deck::iter`], [`Deck::iter_in`], and [`Deck::iter_drawn`].
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
        let bit = self.mask.isolate_lowest_one();
        self.mask &= self.mask - 1;
        Some(card_id(bit))
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
        Some(card_id(1u64 << idx))
    }
}

impl ExactSizeIterator for Iter {}
impl core::iter::FusedIterator for Iter {}

#[inline]
#[allow(clippy::cast_possible_truncation)]
fn card_id(bit: u64) -> u8 {
    // A card bit is always one of the 64 bits in the deck mask.
    bit.trailing_zeros() as u8
}

impl<const N: u8> IntoIterator for &Deck<N> {
    type Item = u8;
    type IntoIter = Iter;

    #[inline]
    fn into_iter(self) -> Iter {
        self.iter()
    }
}

/// Check whether the x86 BMI2 instruction set is available.
///
/// - Short-circuits on `cfg!(target_feature = "bmi2")` for zero runtime cost
///   when the target explicitly enables BMI2.
/// - Caches the CPUID result so repeated checks don't re-execute the
///   serializing instruction.
/// - Works around the Skylake erratum (SKL052) where some steppings report
///   BMI1/BMI2 in CPUID without actually supporting the instructions; on
///   `GenuineIntel` we additionally require AVX, which every real BMI2 chip has.
#[cfg(target_arch = "x86_64")]
#[inline]
fn bmi2_available() -> bool {
    use core::sync::atomic::{AtomicU8, Ordering};

    const UNKNOWN: u8 = 0;
    const NO: u8 = 1;
    const YES: u8 = 2;
    static CACHE: AtomicU8 = AtomicU8::new(UNKNOWN);

    if cfg!(target_feature = "bmi2") {
        return true;
    }

    match CACHE.load(Ordering::Relaxed) {
        YES => return true,
        NO => return false,
        _ => {}
    }

    let detected = detect_bmi2();
    CACHE.store(if detected { YES } else { NO }, Ordering::Relaxed);
    detected
}

#[cfg(target_arch = "x86_64")]
fn detect_bmi2() -> bool {
    // CPUID uses EAX as an input (leaf) and ECX as a secondary input (subleaf).
    // These values are identifier numbers that are used by CPUID instruction to retrieve information about the CPU.
    // The CPU returns answers in EAX, EBX, ECX, and EDX.
    // ref : www.felixcloutier.com/x86/cpuid
    //
    // Set EAX=0 and call CPUID
    // Returns
    // EBX, EDX, ECX (ordered): CPU's manufacturer ID string (12-character ASCII)
    // EAX: Highest Function Parameter (Maximum leaf value) supported by the CPU
    let leaf0 = core::arch::x86_64::__cpuid(0);
    // BMI2 lives in leaf 7 with bit 8 set in EBX
    if leaf0.eax < 7 {
        return false;
    }

    // Set EAX=7, ECX=0 and call CPUID
    // returns
    // EBX, ECX, and EDX: extended feature flags
    // EAX: the maximum ECX value (subleaf) for EAX=7
    let leaf7 = core::arch::x86_64::__cpuid(7);
    // 8th EBX bit indicates BMI2 support
    if leaf7.ebx & (1 << 8) == 0 {
        return false;
    }

    // SKL052: some Skylake steppings set the BMI1/BMI2 CPUID bits without
    // real support; using the instructions then raises #UD. Every genuine
    // BMI2-capable Intel chip also reports AVX, so gate on that.
    let vendor: [u8; 12] = {
        let mut v = [0u8; 12];
        v[0..4].copy_from_slice(&leaf0.ebx.to_ne_bytes());
        v[4..8].copy_from_slice(&leaf0.edx.to_ne_bytes());
        v[8..12].copy_from_slice(&leaf0.ecx.to_ne_bytes());
        v
    };
    if &vendor == b"GenuineIntel" {
        // Set EAX=1 and call CPUID
        // Returns
        // EAX: CPU Signature (stepping, model, microarchitecture codename, family information)
        // EDX and ECX: Feature flags
        // ECX: additional feature flags
        let leaf1 = core::arch::x86_64::__cpuid(1);
        return (leaf1.ecx >> 28) & 1 != 0; // AVX bit
    }

    true
}

#[inline]
pub(crate) fn select_nth_set(mask: u64, k: u32) -> u64 {
    debug_assert!(
        k < mask.count_ones(),
        "k ({k}) must be less than the popcount of the mask ({})",
        mask.count_ones()
    );
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: confirm BMI2 availability - we first check the compiletime feature flag, else the runtime check
        if bmi2_available() {
            return unsafe { core::arch::x86_64::_pdep_u64(1u64 << k, mask) };
        }
    }
    // Portable fallback: clear the lowest set bit k times, then isolate it.
    let mut m = mask;
    for _ in 0..k {
        m &= m - 1; // Brian Kernighan's algorithm
    }
    m.isolate_lowest_one()
}
