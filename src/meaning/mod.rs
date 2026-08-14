/// Card-specific meanings.
#[cfg(feature = "cards")]
pub mod cards;

#[cfg(feature = "cards")]
pub use cards::{CARD_COUNT, FULL_DECK, StdDeck};

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
/// - `from_id(id) -> Self` — the mapping itself; panics on out-of-range or
///   unmapped ids,
/// - `mask(self) -> u64` — the bitmask of every id with this meaning,
/// - `ALL: u64` — the bitmask of every id the meaning covers.
///
/// The returned masks plug straight into [`crate::Deck::contains_any`],
/// [`crate::Deck::contains_all`], [`crate::Deck::count_in`], and
/// `crate::Deck::draw_in`, and compose with `|`, `&`, and `!` in const
/// contexts.
///
/// The mapping is an arbitrary expression over the id, so irregular layouts
/// (jokers, short decks, non-uniform suits) work as long as unmapped ids below
/// `cards` produce an index with no matching variant. `cards` bounds the ids
/// the mapping must handle; ids at or above it are ignored by the generated
/// `mask` but will panic in `from_id`.
///
/// Variants are assigned indices `0, 1, 2, ...` in declaration order. The
/// `from_id` expression must return those exact indices — for example, the
/// first declared variant must map from id `0`.
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
            /// Panics if `id` is out of range (greater than or equal to
            /// `cards`) or if the mapping produces no variant.
            #[inline]
            #[must_use]
            pub const fn from_id(id: u8) -> Self {
                if id >= $cards {
                    panic!("card id out of range");
                }
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
