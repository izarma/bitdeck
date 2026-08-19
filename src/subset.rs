//! Deck-scoped, typed subsets.
//!
//! The [`Subset`] trait marks a type as representing a set of card ids for a
//! specific deck newtype. The [`deck!`] macro generates the newtype together
//! with its classifications (enums) and fixed subsets (unit structs), all
//! scoped to that deck.

/// A subset of a specific deck `D`.
///
/// Implemented by classification enums and fixed-subset unit structs generated
/// by [`deck!`](crate::deck). The deck newtype then accepts any `impl Subset<D>`
/// in its typed subset methods such as `count_subset`.
///
/// The trait is intentionally non-const; for const mask algebra use the raw
/// `u64` API on [`Deck<N>`](crate::Deck).
///
/// Because `Subset` requires [`Copy`], user-defined types that are not `Copy`
/// cannot implement it. This matches the intended use for small, value-like
/// classifications.
///
/// <https://rust-lang.github.io/goals/2024h2/const-traits.html>
pub trait Subset<D>: Copy {
    /// Returns the raw `u64` bitmask of card ids in this subset.
    fn mask(self) -> u64;
}

/// Defines a deck newtype, its classification enums, and its fixed subsets.
///
/// The generated newtype wraps [`Deck<N>`](crate::Deck), implements
/// `Default`, `Deref`, `DerefMut`, and `IntoIterator for &Name`, and adds typed
/// `_subset` methods that accept any `impl Subset<D>`.
///
/// # Syntax
///
/// ```ignore
/// bitdeck::deck! {
///     pub struct Standard = Deck<54>;
///
///     subsets {
///         // Classification enum: each variant maps to a bitmask.
///         pub enum Suit { Clubs, Diamonds, Hearts, Spades }
///         from_id = |id: u8| id / 13;   // must return 0, 1, 2, ...
///         cards = 52;                   // const expression, must be <= 64
///
///         // Fixed subset: a unit struct with an explicit mask.
///         pub struct Jokers {
///             mask = (1u64 << 52) | (1u64 << 53); // const expression
///         }
///     }
/// }
/// ```
///
/// `cards` and fixed-subset `mask` must be **const expressions** because they
/// are evaluated inside generated `const fn`s.
///
/// The `subsets` block may be omitted if you only need the deck newtype.
///
/// Classification enums receive:
/// - `from_id(id: u8) -> Self` — the mapping; panics on out-of-range ids or
///   ids whose `from_id` value does not correspond to a variant.
/// - `mask(self) -> u64` — the bitmask of every mapped card id.
/// - `ALL: u64` — the union of all variant masks.
/// - `impl Subset<Newtype>` so variant values work as typed subsets.
///
/// The `from_id` closure must return the variant's discriminant for each
/// covered id. Variants receive discriminants `0, 1, 2, …` in declaration
/// order, so for default-discriminant enums the closure returns the zero-based
/// index of the variant.
///
/// Fixed unit structs receive:
/// - `mask(self) -> u64` — the explicit mask.
/// - `impl Subset<Newtype>` so the unit value works as a typed subset.
///
/// Generated `Subset` impls cannot be overridden: the inherent `mask()` method
/// and the trait impl are emitted together, and a user-written impl would
/// conflict with the macro-generated one.
///
/// # Example
///
/// ```
/// use bitdeck::{Deck, deck};
///
/// deck! {
///     struct Short = Deck<10>;
///
///     subsets {
///         enum Half { Low, High }
///         from_id = |id: u8| id / 5;
///         cards = 10;
///     }
/// }
///
/// let mut deck = Short::default();
/// assert_eq!(deck.count_subset(Half::Low), 5);
/// ```
#[macro_export]
macro_rules! deck {
    (
        $(#[$deck_meta:meta])*
        $vis:vis struct $Name:ident = Deck<$N:literal>;
        subsets { $($tokens:tt)* }
    ) => {
        $(#[$deck_meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(
            feature = "bevy",
            derive(bevy_ecs::prelude::Component, bevy_reflect::Reflect)
        )]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        $vis struct $Name(pub $crate::Deck<$N>);

        impl core::default::Default for $Name {
            fn default() -> Self {
                Self($crate::Deck::<$N>::default())
            }
        }

        impl core::ops::Deref for $Name {
            type Target = $crate::Deck<$N>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $Name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<'a> core::iter::IntoIterator for &'a $Name {
            type Item = u8;
            type IntoIter = $crate::Iter;
            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        #[allow(dead_code)]
        impl $Name {
            /// Returns an empty deck.
            #[inline]
            #[must_use]
            pub const fn empty() -> Self {
                Self($crate::Deck::<$N>::empty())
            }

            /// Creates a deck from a raw bitmask.
            #[inline]
            #[must_use]
            pub fn from_bits(mask: u64) -> Self {
                Self($crate::Deck::<$N>::from_bits(mask))
            }

            /// Counts how many cards of `subset` remain.
            #[inline]
            #[must_use]
            pub fn count_subset<S: $crate::Subset<$Name>>(&self, s: S) -> u32 {
                self.count_in(s.mask())
            }

            /// Returns the probability that a uniformly random remaining card is
            /// in `subset`.
            #[inline]
            #[must_use]
            pub fn chance_subset<S: $crate::Subset<$Name>>(&self, s: S) -> f64 {
                self.chance(s.mask())
            }

            /// Returns `true` if any card of `subset` remains.
            #[inline]
            #[must_use]
            pub fn contains_any_subset<S: $crate::Subset<$Name>>(&self, s: S) -> bool {
                self.contains_any(s.mask())
            }

            /// Returns `true` if every card of `subset` remains.
            #[inline]
            #[must_use]
            pub fn contains_all_subset<S: $crate::Subset<$Name>>(&self, s: S) -> bool {
                self.contains_all(s.mask())
            }

            /// Returns `true` if every remaining card is in `subset`.
            #[inline]
            #[must_use]
            pub fn is_subset_of_subset<S: $crate::Subset<$Name>>(&self, s: S) -> bool {
                self.is_subset_of(s.mask())
            }

            /// Iterates over the remaining cards in `subset`.
            #[inline]
            #[must_use]
            pub fn iter_subset<S: $crate::Subset<$Name>>(&self, s: S) -> $crate::Iter {
                self.iter_in(s.mask())
            }

            /// Returns the `k`th remaining card in `subset` by card id.
            #[inline]
            #[must_use]
            pub fn nth_subset<S: $crate::Subset<$Name>>(&self, s: S, k: u32) -> Option<u8> {
                self.nth_in(s.mask(), k)
            }

            /// Returns the lowest-id remaining card in `subset`.
            #[inline]
            #[must_use]
            pub fn first_subset<S: $crate::Subset<$Name>>(&self, s: S) -> Option<u8> {
                self.first_in(s.mask())
            }

            /// Returns the highest-id remaining card in `subset`.
            #[inline]
            #[must_use]
            pub fn last_subset<S: $crate::Subset<$Name>>(&self, s: S) -> Option<u8> {
                self.last_in(s.mask())
            }

            /// Removes every remaining card in `subset` and returns the count.
            #[inline]
            pub fn remove_all_subset<S: $crate::Subset<$Name>>(&mut self, s: S) -> u32 {
                self.remove_all(s.mask())
            }

            /// Keeps only the remaining cards in `subset`.
            #[inline]
            pub fn retain_subset<S: $crate::Subset<$Name>>(&mut self, s: S) {
                self.retain(s.mask())
            }

            /// Flips the presence of every card in `subset`.
            #[inline]
            pub fn toggle_all_subset<S: $crate::Subset<$Name>>(&mut self, s: S) {
                self.toggle_all(s.mask())
            }

            /// Draws one random card from `subset`, removing it.
            #[cfg(feature = "rand")]
            #[inline]
            pub fn draw_subset<S: $crate::Subset<$Name>>(
                &mut self,
                rng: &mut impl $crate::__rand::Rng,
                s: S,
            ) -> Option<u8> {
                self.draw_in(rng, s.mask())
            }

            /// Draws up to `count` random cards from `subset` and returns them
            /// as a bitmask.
            #[cfg(feature = "rand")]
            #[inline]
            pub fn draw_subset_mask<S: $crate::Subset<$Name>>(
                &mut self,
                rng: &mut impl $crate::__rand::Rng,
                s: S,
                count: usize,
            ) -> u64 {
                self.draw_in_mask(rng, s.mask(), count)
            }

            /// Draws up to `count` cards from `subset` into `out`.
            #[cfg(all(feature = "rand", feature = "alloc"))]
            #[inline]
            pub fn draw_subset_into<S: $crate::Subset<$Name>>(
                &mut self,
                rng: &mut impl $crate::__rand::Rng,
                s: S,
                count: usize,
                out: &mut $crate::__alloc::Vec<u8>,
            ) -> usize {
                self.draw_in_into(rng, s.mask(), count, out)
            }

            /// Peeks at one random card from `subset` without removing it.
            #[cfg(feature = "rand")]
            #[inline]
            pub fn peek_subset<S: $crate::Subset<$Name>>(
                &self,
                rng: &mut impl $crate::__rand::Rng,
                s: S,
            ) -> Option<u8> {
                self.peek_in(rng, s.mask())
            }

            /// Peeks up to `count` random cards from `subset` and returns them
            /// as a bitmask without removing them.
            #[cfg(feature = "rand")]
            #[inline]
            pub fn peek_subset_mask<S: $crate::Subset<$Name>>(
                &self,
                rng: &mut impl $crate::__rand::Rng,
                s: S,
                count: usize,
            ) -> u64 {
                self.peek_in_mask(rng, s.mask(), count)
            }

            /// Peeks up to `count` cards from `subset` into `out` without
            /// removing them.
            #[cfg(all(feature = "rand", feature = "alloc"))]
            #[inline]
            pub fn peek_subset_into<S: $crate::Subset<$Name>>(
                &self,
                rng: &mut impl $crate::__rand::Rng,
                s: S,
                count: usize,
                out: &mut $crate::__alloc::Vec<u8>,
            ) -> usize {
                self.peek_in_into(rng, s.mask(), count, out)
            }
        }

        $crate::deck! {@subsets $Name $N [] $($tokens)*}
    };

    // Terminal rule: emit all accumulated subset items.
    (@subsets $Name:ident $N:literal [$($gen:tt)*]) => { $($gen)* };

    // Classification enum subset.
    (@subsets $Name:ident $N:literal [$($gen:tt)*]
        $(#[$emeta:meta])*
        $evis:vis enum $E:ident $body:tt
        from_id = |$id:ident : u8| $body_expr:expr;
        cards = $cards:expr;
        $($rest:tt)*
    ) => {
        $crate::deck! {@subsets $Name $N [
            $($gen)*
            $(#[$emeta])*
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            $evis enum $E $body
            $crate::deck! {@enum_impl $E $body $id ($body_expr) ($cards)}
            impl $crate::Subset<$Name> for $E {
                // Resolves to the inherent `E::mask`, not the trait method.
                fn mask(self) -> u64 { self.mask() }
            }
        ] $($rest)*}
    };

    // Fixed unit-struct subset.
    (@subsets $Name:ident $N:literal [$($gen:tt)*]
        $(#[$smeta:meta])*
        $svis:vis struct $S:ident { mask = $mask_expr:expr; }
        $($rest:tt)*
    ) => {
        $crate::deck! {@subsets $Name $N [
            $($gen)*
            $(#[$smeta])*
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            $svis struct $S;
            impl $S {
                /// Returns the bitmask of this fixed subset.
                #[inline]
                #[must_use]
                pub const fn mask(self) -> u64 { $mask_expr }
            }
            impl $crate::Subset<$Name> for $S {
                // Resolves to the inherent `S::mask`, not the trait method.
                fn mask(self) -> u64 { self.mask() }
            }
            const _: () = assert!(
                $mask_expr & !$crate::full_mask::<$N>() == 0,
                concat!("fixed subset `", stringify!($S), "` has bits above the deck size")
            );
        ] $($rest)*}
    };

    // Inherent impl for a classification enum: from_id, mask, ALL.
    (@enum_impl $E:ident { $($(#[$vmeta:meta])* $variant:ident),* $(,)? } $id:ident ($body_expr:expr) ($cards:expr)) => {
        #[allow(dead_code)]
        impl $E {
            /// Returns the meaning of card `id`.
            ///
            /// # Panics
            ///
            /// Panics if `id` is out of range (greater than or equal to
            /// `cards`) or if the mapping produces no variant.
            #[inline]
            #[must_use]
            pub const fn from_id(id: u8) -> Self {
                assert!($cards <= 64, "deck size must fit in a u64 bitmask");
                if id >= $cards {
                    core::panic!(concat!("card id out of range for ", stringify!($E)));
                }
                let index = { let $id = id; $body_expr };
                $( if index == Self::$variant as u8 { return Self::$variant; } )*
                core::panic!(concat!("card id has no ", stringify!($E)));
            }

            /// Returns the bitmask of every card id with this meaning,
            /// computed by inverting the id mapping at compile time.
            #[inline]
            #[must_use]
            pub const fn mask(self) -> u64 {
                assert!($cards <= 64, "deck size must fit in a u64 bitmask");
                let target = self as u8;
                let mut mask = 0u64;
                let mut id = 0u8;
                while id < $cards {
                    let index = { let $id = id; $body_expr };
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
