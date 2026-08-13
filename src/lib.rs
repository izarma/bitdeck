#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "rand")]
use rand::{Rng, RngExt};

/// Standard-card suits, ranks, colors, and predefined masks.
#[cfg(feature = "cards")]
pub mod cards;

/// Bitmask with all [`CARD_COUNT`] bits set; the initial state of a full [`Deck`].
pub const FULL_DECK: u64 = full_mask::<CARD_COUNT>();

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
/// See [`StdDeck`] for the usual 52-card deck plus two jokers.
///
/// Subset APIs accept a `u64` mask of card ids, typically built with
/// [`stride_mask`] or [`meanings!`]. Bits at or above `N` are ignored by
/// subset queries and draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Resource))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Deck<const N: u8> {
    mask: u64,
}

/// Number of cards in a [`StdDeck`].
pub const CARD_COUNT: u8 = 54;
/// A standard 52-card deck plus two jokers (`Deck<54>`).
pub type StdDeck = Deck<CARD_COUNT>;

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
    /// use bitdeck::StdDeck;
    ///
    /// let deck = StdDeck::default();
    /// assert_eq!(StdDeck::from_bits(deck.as_bits()), deck);
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
    ///
    /// Bits at or above `N` are ignored.
    #[inline]
    pub const fn insert_all(&mut self, mask: u64) {
        self.mask |= mask & full_mask::<N>();
    }

    /// Removes every remaining card selected by `mask` and returns how many
    /// were removed.
    ///
    /// Bits at or above `N` are ignored.
    #[inline]
    pub const fn remove_all(&mut self, mask: u64) -> u32 {
        let removed = self.count_in(mask);
        self.mask &= !mask;
        removed
    }

    /// Keeps only the remaining cards selected by `mask`.
    ///
    /// Bits at or above `N` are ignored.
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
    /// use bitdeck::StdDeck;
    ///
    /// assert_eq!(StdDeck::empty().iter().count(), 0);
    /// assert_eq!(StdDeck::default().iter().count(), 54);
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
        if self.mask == 0 {
            return None;
        }
        let k = rng.random_range(0..self.remaining());
        let bit = select_nth_set(self.mask, k);
        self.mask &= !bit;
        Some(card_id(bit))
    }

    /// Draws one random card from a subset of the deck, removing it.
    ///
    /// Returns `None` if no card of the subset remains. Bits at or above `N`
    /// in `mask` are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::{StdDeck, stride_mask};
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// const HEARTS: u64 = stride_mask(26, 1, 13);
    ///
    /// let mut deck = StdDeck::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    ///
    /// let card = deck.draw_in(&mut rng, HEARTS).unwrap();
    /// assert_eq!(card / 13, 2); // every drawn card is a heart
    /// assert_eq!(deck.count_in(HEARTS), 12);
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
    /// use bitdeck::StdDeck;
    /// use rand::{SeedableRng, rngs::SmallRng};
    ///
    /// let mut deck = StdDeck::default();
    /// let mut rng = SmallRng::seed_from_u64(420);
    /// let mut hand = Vec::new();
    ///
    /// // Ask for 5 cards and check the count to detect a short deck.
    /// let drawn = deck.draw_into(&mut rng, 5, &mut hand);
    /// if drawn < 5 {
    ///     // Reshuffle, error, or play the partial hand.
    /// }
    /// ```
    #[cfg(feature = "rand")]
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
    /// use bitdeck::{StdDeck, stride_mask};
    ///
    /// const KINGS: u64 = stride_mask(12, 13, 4);
    ///
    /// let mut deck = StdDeck::default();
    /// assert!(deck.contains_any(KINGS));
    /// ```
    #[inline]
    #[must_use]
    pub const fn contains_any(&self, mask: u64) -> bool {
        self.mask & mask != 0
    }

    /// Returns `true` if every card of the subset `mask` is still in the deck.
    #[inline]
    #[must_use]
    pub const fn contains_all(&self, mask: u64) -> bool {
        self.mask & mask == mask
    }

    /// Returns how many cards of the subset `mask` are still in the deck.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitdeck::{StdDeck, stride_mask};
    ///
    /// const HEARTS: u64 = stride_mask(26, 1, 13);
    ///
    /// let deck = StdDeck::default();
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
}

/// Bitmask of `count` card ids starting at `start`, each `step` apart:
/// `{start, start + step, ..., start + (count - 1) * step}`.
///
/// Masks compose with `|`, `&`, and `!` in const contexts, so irregular
/// subsets are unions of regular ones.
///
/// # Panics
///
/// Panics if any selected bit index exceeds 63.
///
/// ```
/// use bitdeck::stride_mask;
///
/// const HEARTS: u64 = stride_mask(26, 1, 13);
/// const KINGS: u64 = stride_mask(12, 13, 4);
///
/// assert_eq!(HEARTS.count_ones(), 13);
/// assert_eq!(KINGS.count_ones(), 4);
/// // The king of hearts is the only card in both.
/// assert_eq!((HEARTS & KINGS).trailing_zeros(), 38);
/// ```
#[must_use]
pub const fn stride_mask(start: u8, step: u8, count: u8) -> u64 {
    let mut mask = 0u64;
    let mut i = 0u8;
    while i < count {
        let index = start + step * i;
        assert!(index <= 63, "selected card id exceeds bitmask size");
        mask |= 1u64 << index;
        i += 1;
    }
    mask
}

/// Defines a card meaning from an enum classifying card ids and derives all
/// bitmask machinery from a single mapping expression.
///
/// You write the enum and how a card id maps to a variant; the macro
/// generates, for each variant, the bitmask of all card ids that map to it
/// (computed at compile time by inverting the mapping), plus:
///
/// - `from_id(id) -> Self` — the mapping itself; panics on unmapped ids,
/// - `mask(self) -> u64` — the bitmask of every id with this meaning,
/// - `ALL: u64` — the bitmask of every id the meaning covers.
///
/// The returned masks plug straight into [`Deck::contains_any`],
/// [`Deck::contains_all`], [`Deck::count_in`], and [`Deck::draw_in`], and
/// compose with `|`, `&`, and `!` in const contexts.
///
/// The mapping is an arbitrary expression over the id, so irregular layouts
/// (jokers, short decks, non-uniform suits) work as long as unmapped ids
/// produce an index with no matching variant. `cards` bounds the ids the
/// mapping must handle; ids at or above it are ignored.
///
/// # Examples
///
/// ```
/// use bitdeck::{Deck, meanings};
///
/// meanings! {
///     pub enum Half {
///         Low,
///         High,
///     }
///     from_id = |id: u8| id / 5;
///     cards = 10;
/// }
///
/// let mut deck = Deck::<10>::default();
/// assert_eq!(deck.count_in(Half::Low.mask()), 5);
/// assert_eq!(Half::from_id(7), Half::High);
///
/// deck.remove(0);
/// assert!(deck.contains_any(Half::Low.mask()));
/// assert!(!deck.contains_all(Half::ALL));
/// ```
#[macro_export]
macro_rules! meanings {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident
            ),* $(,)?
        }
        from_id = |$id:ident : u8| $body:expr;
        cards = $cards:expr;
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )*
        }

        impl $name {
            /// Returns the meaning of card `id`.
            ///
            /// # Panics
            ///
            /// Panics if `id` maps to no variant.
            #[inline]
            #[must_use]
            pub const fn from_id(id: u8) -> Self {
                let index = { let $id = id; $body };
                $( if index == Self::$variant as u8 { return Self::$variant; } )*
                panic!(concat!("card id has no ", stringify!($name)))
            }

            /// Returns the bitmask of every card id with this meaning,
            /// computed by inverting the id mapping at compile time.
            #[inline]
            #[must_use]
            pub const fn mask(self) -> u64 {
                let target = self as u8;
                let mut mask = 0u64;
                let mut id = 0u8;
                while id < $cards {
                    let index = { let $id = id; $body };
                    if index == target {
                        mask |= 1u64 << id;
                    }
                    id += 1;
                }
                mask
            }

            /// Bitmask of every card id this meaning covers.
            pub const ALL: u64 = 0u64 $( | Self::$variant.mask() )*;
        }
    };
}

/// Iterator over the cards remaining in a [`Deck`], in ascending card-id
/// order. Created by [`Deck::iter`].
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

#[inline]
fn select_nth_set(mask: u64, k: u32) -> u64 {
    debug_assert!(
        k < mask.count_ones(),
        "k ({k}) must be less than the popcount of the mask ({})",
        mask.count_ones()
    );
    #[cfg(target_arch = "x86_64")]
    {
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
    m.isolate_lowest_one()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rand")]
    use rand::{SeedableRng, rngs::SmallRng};

    #[cfg(feature = "rand")]
    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(0x1234_5678_9ABC_DEF0)
    }

    #[test]
    #[should_panic(expected = "selected card id exceeds bitmask size")]
    fn stride_mask_panics_when_index_exceeds_bitmask() {
        let _ = stride_mask(60, 2, 3); // 60, 62, then 64 > 63
    }

    #[test]
    #[should_panic(expected = "card id 8 out of range")]
    fn insert_panics_on_out_of_range_card() {
        Deck::<8>::default().insert(8);
    }

    #[test]
    #[should_panic(expected = "card id 8 out of range")]
    fn remove_panics_on_out_of_range_card() {
        Deck::<8>::default().remove(8);
    }

    #[test]
    #[should_panic(expected = "deck size must fit in a u64 bitmask")]
    fn full_mask_panics_above_64() {
        let _ = full_mask::<65>();
    }

    #[test]
    #[should_panic(expected = "card id 4 out of range")]
    fn contains_panics_on_out_of_range_card() {
        let _ = Deck::<4>::default().contains(4);
    }

    #[test]
    #[cfg_attr(not(debug_assertions), ignore)]
    #[should_panic(expected = "bits at or above deck size are not cards")]
    fn from_bits_panics_on_bits_above_size_in_debug() {
        // In release builds high bits are silently masked, so the test is ignored.
        let _ = Deck::<4>::from_bits(u64::MAX);
    }

    #[test]
    fn default_remaining_and_full_mask() {
        assert_eq!(Deck::<1>::default().remaining(), 1);
        assert_eq!(Deck::<13>::default().remaining(), 13);
        assert_eq!(Deck::<64>::default().remaining(), 64);
        assert_eq!(Deck::<13>::empty().remaining(), 0);
        assert_eq!(full_mask::<64>(), u64::MAX);
        assert_eq!(FULL_DECK, full_mask::<CARD_COUNT>());
    }

    #[test]
    fn as_bits_round_trips_generic() {
        let mut deck = Deck::<10>::default();
        deck.remove(3);
        deck.remove(7);
        let mask = deck.as_bits();
        let restored = Deck::<10>::from_bits(mask);
        assert_eq!(restored, deck);
    }

    #[test]
    fn iter_order_double_ended_and_into_iter() {
        let mut deck = Deck::<5>::default();
        deck.remove(2);

        let mut iter = deck.iter();
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.collect::<Vec<_>>(), vec![1, 3]);

        let collected: Vec<_> = (&deck).into_iter().collect();
        assert_eq!(collected, vec![0, 1, 3, 4]);
    }

    #[test]
    #[cfg(feature = "rand")]
    fn draw_all_cards_leaves_deck_empty() {
        let mut deck = Deck::<32>::default();
        let mut rng = test_rng();
        let mut drawn = Vec::with_capacity(32);
        while let Some(card) = deck.draw(&mut rng) {
            drawn.push(card);
        }
        assert!(deck.is_empty());
        drawn.sort_unstable();
        assert_eq!(drawn, (0u8..32).collect::<Vec<_>>());
    }

    #[test]
    #[cfg(feature = "rand")]
    fn draw_from_full_64_deck_exercises_top_bit() {
        let mut deck = Deck::<64>::default();
        let mut rng = test_rng();
        let mut drawn = Vec::with_capacity(64);
        while let Some(card) = deck.draw(&mut rng) {
            drawn.push(card);
        }
        assert!(deck.is_empty());
        drawn.sort_unstable();
        assert_eq!(drawn, (0u8..64).collect::<Vec<_>>());
    }

    #[test]
    fn iter_is_fused_after_exhaustion() {
        let mut iter = Deck::<4>::empty().iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);

        let mut iter = Deck::<2>::default().iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        // Fused: stays `None`, from both ends.
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_as_transparent_u64() {
        let mut deck = Deck::<54>::default();
        deck.remove(0);
        deck.remove(53);
        let text = ron::to_string(&deck).unwrap();
        // `serde(transparent)`: the deck serializes as its raw mask.
        assert_eq!(text, deck.as_bits().to_string());
        let restored: Deck<54> = ron::from_str(&text).unwrap();
        assert_eq!(restored, deck);
    }

    #[cfg(feature = "bevy")]
    #[test]
    fn deck_is_a_usable_bevy_resource() {
        use bevy_ecs::prelude::*;
        fn assert_resource<T: Resource>() {}
        assert_resource::<StdDeck>();
        assert_resource::<Deck<10>>();

        let mut world = World::new();
        world.insert_resource(StdDeck::default());
        assert_eq!(*world.resource::<StdDeck>(), StdDeck::default());
    }

    #[test]
    #[cfg(feature = "rand")]
    fn draws_are_deterministic_with_seed() {
        let (mut a, mut b) = (Deck::<16>::default(), Deck::<16>::default());
        let (mut rng_a, mut rng_b) = (test_rng(), test_rng());
        for _ in 0..10 {
            assert_eq!(a.draw(&mut rng_a), b.draw(&mut rng_b));
        }
        for _ in 0..4 {
            assert_eq!(a.draw_in(&mut rng_a, SPREAD), b.draw_in(&mut rng_b, SPREAD));
        }
    }

    #[test]
    #[cfg(feature = "rand")]
    fn restock_restores_a_full_deck() {
        let mut deck = Deck::<16>::default();
        let _ = deck.draw_into(&mut test_rng(), 10, &mut Vec::new());
        deck.restock();
        assert_eq!(deck.remaining(), 16);
        assert_eq!(deck, Deck::<16>::default());
    }

    #[test]
    #[cfg(feature = "rand")]
    fn draw_into_clears_buffer_and_draws_requested_count() {
        let mut deck = Deck::<16>::default();
        let mut out = vec![255, 254];
        let drawn = deck.draw_into(&mut test_rng(), 5, &mut out);
        assert_eq!(drawn, 5);
        assert_eq!(out.len(), 5);
        assert_eq!(deck.remaining(), 11);
    }

    #[test]
    #[cfg(feature = "rand")]
    fn draw_into_stops_when_deck_is_empty() {
        let mut deck = Deck::<16>::default();
        let mut out = Vec::new();
        let drawn = deck.draw_into(&mut test_rng(), 100, &mut out);
        assert_eq!(drawn, 16);
        assert_eq!(out.len(), 16);
        assert!(deck.is_empty());
    }

    #[test]
    fn bulk_mutations_ignore_out_of_range_bits_and_report_removals() {
        let mut deck = Deck::<8>::empty();
        deck.insert_all(0b1_0000_1010);
        assert_eq!(deck.as_bits(), 0b1010);

        assert_eq!(deck.remove_all(0b1_0000_1001), 1);
        assert_eq!(deck.as_bits(), 0b0010);

        deck.insert_all(0b0111_0100);
        deck.retain(0b1_0011_0110);
        assert_eq!(deck.as_bits(), 0b0011_0110);
    }

    #[test]
    fn drawn_card_introspection_and_iteration_are_complementary() {
        let mut deck = Deck::<8>::default();
        assert_eq!(deck.drawn_mask(), 0);
        assert_eq!(deck.drawn_count(), 0);

        deck.remove_all(0b1010_0101);
        assert_eq!(deck.drawn_mask(), 0b1010_0101);
        assert_eq!(deck.drawn_count(), 4);
        assert_eq!(deck.iter_drawn().collect::<Vec<_>>(), vec![0, 2, 5, 7]);
        assert_eq!(deck.iter_in(0b0011_1110).collect::<Vec<_>>(), vec![1, 3, 4]);
    }

    #[test]
    fn deterministic_selection_does_not_mutate_the_deck() {
        let deck = Deck::<8>::from_bits(0b1011_0010);
        assert_eq!(deck.first(), Some(1));
        assert_eq!(deck.last(), Some(7));
        assert_eq!(deck.nth(0), Some(1));
        assert_eq!(deck.nth(1), Some(4));
        assert_eq!(deck.nth(3), Some(7));
        assert_eq!(deck.nth(4), None);
        assert_eq!(deck, Deck::<8>::from_bits(0b1011_0010));

        let empty = Deck::<8>::empty();
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);
        assert_eq!(empty.nth(0), None);
    }

    #[test]
    #[cfg(feature = "rand")]
    fn random_selection_does_not_mutate_the_deck() {
        let deck = Deck::<8>::from_bits(0b1011_0010);
        let mut rng = test_rng();
        let before = deck;
        assert!(matches!(deck.peek(&mut rng), Some(1 | 4 | 5 | 7)));
        assert!(matches!(deck.peek_in(&mut rng, 0b0011_0000), Some(4 | 5)));
        assert_eq!(deck.peek_in(&mut rng, 0b0000_1000), None);
        assert_eq!(deck, before);

        let empty = Deck::<8>::empty();
        assert_eq!(empty.peek(&mut rng), None);
    }

    #[test]
    fn chance_is_the_fraction_of_remaining_cards_in_a_subset() {
        let deck = Deck::<8>::from_bits(0b1011_0010);
        assert_eq!(deck.chance(0b0011_0000), 0.5);
        assert_eq!(deck.chance(0b1_0000_0000), 0.0);
        assert!(Deck::<8>::empty().chance(u64::MAX).is_nan());
    }

    #[test]
    fn insert_contains_remove_roundtrip() {
        let mut deck = Deck::<8>::empty();
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
    fn select_nth_set_selects_kth_set_bit() {
        let mask = 0b1010_1100u64;
        for (k, expected) in [
            (0, 0b0000_0100),
            (1, 0b0000_1000),
            (2, 0b0010_0000),
            (3, 0b1000_0000),
        ] {
            assert_eq!(select_nth_set(mask, k), expected);
        }

        let full = full_mask::<16>();
        for k in [0, 1, 5, 14, 15] {
            assert_eq!(select_nth_set(full, k), 1u64 << k);
        }
    }

    // meanings

    const LOW: u64 = stride_mask(0, 1, 4); // ids 0, 1, 2, 3
    const SPREAD: u64 = stride_mask(1, 3, 4); // ids 1, 4, 7, 10
    const TAIL: u64 = stride_mask(14, 1, 2); // ids 14, 15

    #[test]
    fn stride_mask_builds_expected_bits() {
        assert_eq!(LOW, 0b1111);
        assert_eq!(SPREAD, (1 << 1) | (1 << 4) | (1 << 7) | (1 << 10));
        assert_eq!(TAIL, 0b11 << 14);
        // Masks compose with the usual bitwise operators.
        let union = LOW | TAIL;
        assert_eq!(union.count_ones(), 6);
        assert_eq!((LOW & SPREAD).count_ones(), 1); // only id 1
        assert_eq!(full_mask::<14>() | TAIL, full_mask::<16>());
    }

    #[test]
    fn mask_queries_track_subset_membership() {
        let mut deck = Deck::<16>::default();
        assert!(deck.contains_any(LOW));
        assert!(deck.contains_all(SPREAD));
        assert_eq!(deck.count_in(SPREAD), 4);

        // Queries ignore mask bits at or above N.
        assert_eq!(deck.count_in(u64::MAX), 16);

        deck.remove(1);
        assert!(deck.contains_any(SPREAD));
        assert!(!deck.contains_all(SPREAD));
        assert_eq!(deck.count_in(SPREAD), 3);

        for id in 0..4 {
            deck.remove(id);
        }
        assert!(!deck.contains_any(LOW));
        assert_eq!(deck.count_in(LOW), 0);
        // Id 1 was in both subsets, so only 4 distinct cards were removed.
        assert_eq!(deck.remaining(), 16 - 4);
    }

    #[test]
    #[cfg(feature = "rand")]
    fn draw_in_draws_only_from_the_subset_until_it_runs_out() {
        let mut deck = Deck::<16>::default();
        let mut rng = test_rng();
        let mut drawn = Vec::new();
        while let Some(card) = deck.draw_in(&mut rng, SPREAD) {
            assert_ne!(SPREAD & (1 << card), 0);
            drawn.push(card);
        }
        assert_eq!(drawn.len(), 4);
        // Everything else is untouched.
        assert_eq!(deck.remaining(), 12);
        assert!(deck.contains_all(TAIL));
    }

    meanings! {
        /// Which half of a 16-card deck an id falls in.
        enum Half {
            /// Ids 0..8.
            Low,
            /// Ids 8..16.
            High,
        }
        from_id = |id: u8| id / 8;
        cards = 16;
    }

    meanings! {
        /// Edge id or middle id?
        enum Zone {
            /// Ids 1..15.
            Middle,
            /// Ids 0 and 15.
            Edge,
        }
        from_id = |id: u8| if id == 0 || id == 15 { 1 } else { 0 };
        cards = 16;
    }

    #[test]
    fn meanings_derives_masks_by_inverting_the_mapping() {
        assert_eq!(Half::High.mask(), stride_mask(8, 1, 8));
        assert_eq!(Half::Low.mask(), stride_mask(0, 1, 8));
        assert_eq!(Half::ALL, full_mask::<16>());
        assert_eq!(Zone::Edge.mask(), (1 << 15) | 1);
        assert_eq!(Zone::ALL, full_mask::<16>());
    }

    #[test]
    fn meanings_from_id_round_trips_through_masks() {
        for id in 0..16 {
            let half = Half::from_id(id);
            assert_ne!(half.mask() & (1 << id), 0);
        }
        assert_eq!(Half::from_id(9), Half::High);
        assert_eq!(Zone::from_id(15), Zone::Edge);
        assert_eq!(Zone::from_id(7), Zone::Middle);
    }

    #[test]
    #[should_panic(expected = "card id has no Half")]
    fn meanings_from_id_panics_on_unmapped_id() {
        let _ = Half::from_id(16); // beyond the covered ids
    }

    #[test]
    fn meanings_plug_into_deck_queries() {
        let deck = Deck::<16>::default();
        assert_eq!(deck.count_in(Half::High.mask()), 8);
        assert!(deck.contains_all(Zone::Edge.mask()));
    }

    #[test]
    #[cfg(feature = "rand")]
    fn meanings_plug_into_random_draws() {
        let mut deck = Deck::<16>::default();
        let mut rng = test_rng();
        let edge = deck.draw_in(&mut rng, Zone::Edge.mask()).unwrap();
        assert!(edge == 0 || edge == 15);
        assert_eq!(deck.count_in(Zone::Edge.mask()), 1);
    }
}
