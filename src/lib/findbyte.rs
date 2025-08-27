use core::{convert::TryInto, ops::{self, BitAnd, BitOr}};

use crate::{core::Parser, number::NumLike, slicelike::SliceLike};

/// One unit of "work". In this case `usize` will process 8 bytes
/// at a time on a 64-bit CPU (or 4 bytes on 32-bit).
pub type Work = usize;

/// 0x0101_0101...
pub const LOW_BITS: Work = Work::MAX / 255;

/// 0x8080_8080...
pub const HIGH_BITS: Work = LOW_BITS << 7;

/// Trait for types that can be used for finding a byte in a wider
/// integer type.
///
/// Use functions [`eq`],  [`ne`], [`lt`], [`gt`] to generate finders.
pub trait ByteFinder: Copy {
    /// Produce an intermediate format used for confirming existence
    /// of byte. The result will be applied to `& 0x808080...` as a final
    /// step.
    fn intermediate(self, haystack: Work) -> Work;

    /// The "traditional" comparison. Used when a full work unit cannot
    /// be obtained.
    fn slow_cmp(self, other: u8)  -> bool;
}

/// A wrapper for combining two [`ByteFinder`] via logic OR.
#[derive(Clone, Copy)]
pub struct OrByte<A: ByteFinder, B: ByteFinder>(pub A, pub B);

/// A wrapper for combining two [`ByteFinder`] via logic AND.
#[derive(Clone, Copy)]
pub struct AndByte<A: ByteFinder, B: ByteFinder>(pub A, pub B);

macro_rules! impl_finder_for_combinator {
    ($id:ident, $bit_op:tt, $logic_op:tt) => {
        impl<A: ByteFinder, B: ByteFinder> ByteFinder for $id<A, B> {
            #[inline]
            fn intermediate(self, haystack: Work) -> Work {
                self.0.intermediate(haystack) $bit_op self.1.intermediate(haystack)
            }

            #[inline]
            fn slow_cmp(self, haystack: u8) -> bool {
                self.0.slow_cmp(haystack) $logic_op self.1.slow_cmp(haystack)
            }
        }
    };
}

impl_finder_for_combinator!(OrByte, |, ||);
impl_finder_for_combinator!(AndByte, &, &&);

macro_rules! impl_bitop_for_finder {
    ($id:ident, $bitop:ident, $bitopf:ident, $output:ident) => {
        impl<A: ByteFinder> $bitop<A> for $id {
            type Output = $output<Self, A>;

            #[inline(always)]
            fn $bitopf(self, rhs: A) -> Self::Output {
                $output(self, rhs)
            }
        }
    };
}

impl_bitop_for_finder!(EqByte, BitOr, bitor, OrByte);
impl_bitop_for_finder!(NeByte, BitOr, bitor, OrByte);
impl_bitop_for_finder!(LtByte, BitOr, bitor, OrByte);
impl_bitop_for_finder!(GtByte, BitOr, bitor, OrByte);
impl_bitop_for_finder!(EqByte, BitAnd, bitand, AndByte);
impl_bitop_for_finder!(NeByte, BitAnd, bitand, AndByte);
impl_bitop_for_finder!(LtByte, BitAnd, bitand, AndByte);
impl_bitop_for_finder!(GtByte, BitAnd, bitand, AndByte);

macro_rules! impl_bitop_for_combinator {
    ($id:ident, $bitop:ident, $bitopf:ident) => {
        impl<A: ByteFinder, B: ByteFinder, C: ByteFinder> $bitop<C> for $id<A, B> {
            type Output = $id<A, $id<B, C>>;

            #[inline(always)]
            fn $bitopf(self, rhs: C) -> Self::Output {
                $id(self.0, $id(self.1, rhs))
            }
        }
    };
}

impl_bitop_for_combinator!(OrByte, BitOr, bitor);
impl_bitop_for_combinator!(OrByte, BitAnd, bitand);
impl_bitop_for_combinator!(AndByte, BitOr, bitor);
impl_bitop_for_combinator!(AndByte, BitAnd, bitand);

/// A wrapper used for finding a byte that is equal to
/// the wrappee.
#[derive(Clone, Copy)]
pub struct EqByte(pub u8);

/// A wrapper used for finding a byte that is smaller than
/// the wrappee.
#[derive(Clone, Copy)]
pub struct LtByte(pub u8);

/// A wrapper used for finding a byte that is greater than
/// the wrappee.
#[derive(Clone, Copy)]
pub struct GtByte(pub u8);

/// A wrapper used for finding a byte that is not equal to
/// the wrappee.
#[derive(Clone, Copy)]
pub struct NeByte(pub u8);

impl ByteFinder for EqByte {
    #[inline]
    fn intermediate(self, haystack: Work) -> Work {
        let to_find = haystack ^ (self.0 as Work * LOW_BITS);
        to_find.wrapping_sub(LOW_BITS) & !to_find
    }

    #[inline(always)]
    fn slow_cmp(self, other: u8) -> bool {
        other == self.0
    }
}

impl ByteFinder for NeByte {
    #[inline]
    fn intermediate(self, haystack: Work) -> Work {
        // Non-equality is obtained by toggling the high bits of
        // equality.
        EqByte(self.0).intermediate(haystack) ^ HIGH_BITS
    }

    #[inline(always)]
    fn slow_cmp(self, other: u8) -> bool {
        other != self.0
    }
}

impl ByteFinder for LtByte {
    #[inline]
    fn intermediate(self, haystack: Work) -> Work {
        haystack.wrapping_sub(LOW_BITS * self.0 as Work) & !haystack
    }

    #[inline(always)]
    fn slow_cmp(self, other: u8) -> bool {
        other < self.0
    }
}

impl ByteFinder for GtByte {
    #[inline]
    fn intermediate(self, haystack: Work) -> Work {
        let mask = LOW_BITS * self.0 as Work;
        mask.wrapping_sub(haystack) & !mask
    }

    #[inline(always)]
    fn slow_cmp(self, other: u8) -> bool {
        other > self.0
    }
}

/// Return a byte finder representing `== b`.
#[inline(always)]
pub const fn eq(b: u8) -> EqByte {
    EqByte(b)
}

/// Return a byte finder representing `!= b`.
#[inline(always)]
pub const fn ne(b: u8) -> NeByte {
    NeByte(b)
}

/// Return a byte finder representing `< b`.
#[inline(always)]
pub const fn lt(b: u8) -> LtByte {
    LtByte(b)
}

/// Return a byte finder representing `> b`.
#[inline(always)]
pub const fn gt(b: u8) -> GtByte {
    GtByte(b)
}

/// Helper function for performing the byte search and returning the
/// result along with its position.
#[inline]
fn get_byte_pos<I, B>(input: I, finder: B) -> Option<(u8, I::Idx)>
    where I: SliceLike + AsRef<[u8]>, B: ByteFinder {
    let mut pos = 0;
    let bytes = input.as_ref();

    let mut chunks = bytes.chunks_exact(Work::SIZE);
    for chunk in chunks.by_ref() {
        let val = Work::from_le_bytes(chunk.try_into().unwrap());
        let present = finder.intermediate(val) & HIGH_BITS;

        if present != 0 {
            pos += (present.trailing_zeros() / u8::BITS) as usize;

            // Inlining this rather than using a labeled break yields slightly
            // better performance.
            return Some((bytes[pos], input.slice_idx_from_offset(pos)))
        }

        pos += Work::SIZE;
    }

    pos += chunks.remainder().iter().position(|x| finder.slow_cmp(*x))?;
    Some((bytes[pos], input.slice_idx_from_offset(pos)))
}

/// Find a single byte in an input that can be represented as a
/// contiguous area of bytes. It will process multiple bytes at
/// a time, more specifically `usize::BITS / 8` bytes.
///
/// When searching for multiple individual bytes, this is likely faster
/// than using [`until`](crate::parsers::until), or using
/// [`item_if`](crate::parsers::item_if) together with
/// [`many`](crate::combinators::many)
///
/// Available finders are:
/// - [`eq(x)`](eq): Search for a byte via equality, i.e. `eq(10)`
///             will search for a byte equal to 10.
/// - [`ne(x)`](ne): Search for a byte via inequality, i.e. `ne(10)`
///             will search for a byte not equal to 10.
/// - [`lt(x)`](lt): Search for a byte via "less than", i.e. `lt(10)`
///             will search for a byte smaller than 10.
/// - [`gt(x)`](gt): Search for a byte via "greater than", i.e. `gt(10)`
///             will search for a byte greater than 10.
///
/// Arguments can be combined with '|' or '&' to search for muliple bytes
/// simultaneously.
///
/// Note: When searching in an UTF-8 string, it is not safe to search
/// for non-ASCII bytes,
///
/// ### Consuming
/// If `consume_result` is:
///   - `true`: all items until and including the match.
///   - `false`: all items until the match.
///
/// ### Arguments
/// * `finder` - the [`ByteFinder`].
/// * `consume_result` - whether the matching byte should be consumed.
///
/// ### Example:
/// ```
/// use anpa::core::*;
/// use anpa::findbyte::*;
///
/// // Find ascii `"`, `\` or control character.
/// let p = find_byte(eq(b'"') | eq(b'\\') | lt(0x20), true);
///
/// let input1 = "abcd\"";
/// let input2 = "ab\\cd";
/// let input3 = "a\nbcd";
///
/// assert_eq!(parse(p, input1).result, Some(b'"'));
/// assert_eq!(parse(p, input2).result, Some(b'\\'));
/// assert_eq!(parse(p, input3).result, Some(b'\n'));
/// ```
#[inline]
pub const fn find_byte<I, S>(finder: impl ByteFinder, consume_result: bool) -> impl Parser<I, u8, S>
    where I: SliceLike + AsRef<[u8]> {
    create_parser!(s, {
        let (res, pos) = get_byte_pos(s.input, finder)?;
        s.input = s.input.slice_from(pos + consume_result.into());
        Some(res)
    })
}

/// Parse until one byte matches in an input that can be represented as a
/// contiguous area of bytes. It will process multiple bytes at
/// a time, more specifically `usize::BITS / 8` bytes.
///
/// When searching for multiple individual bytes, this is likely faster
/// than using [`until`](crate::parsers::until).
///
/// Note: When searching in an UTF-8 string, it is not safe to search
/// for non-ASCII bytes,
///
/// ### Consuming
/// If `consume_result` is:
///   - `true`: all items until and including the match.
///   - `false`: all items until the match.
///
/// ### Arguments
/// * `finder` - the [`ByteFinder`].
/// * `include_result` - whether the matching byte should be incuded
///                      in the result.
/// * `consume_result` - whether the matching byte should be consumed.
///
/// ### Example:
/// ```
/// use anpa::core::*;
/// use anpa::findbyte::*;
///
/// // Find ascii `"`, `\` or control character.
/// let p = until_byte(eq(b'"') | eq(b'\\') | lt(0x20), false, true);
///
/// let input1 = "abcd\"";
/// let input2 = "ab\\cd";
/// let input3 = "a\nbcd";
///
/// assert_eq!(parse(p, input1).result, Some("abcd"));
/// assert_eq!(parse(p, input2).result, Some("ab"));
/// assert_eq!(parse(p, input3).result, Some("a"));
/// ```
#[inline]
pub const fn until_byte<I, S>(finder: impl ByteFinder,
                              include_result: bool,
                              consume_result: bool) -> impl Parser<I, I, S>
    where I: SliceLike + AsRef<[u8]> {
    create_parser!(s, {
        let (_, pos) = get_byte_pos(s.input, finder)?;
        let res = s.input.slice_to(pos + include_result.into());
        s.input = s.input.slice_from(pos + consume_result.into());
        Some(res)
    })
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, findbyte::{eq, find_byte, gt, lt, ne, ByteFinder}};

    #[test]
    fn less_than() {
        let arr = [9, 8, 7, 6, 5, 4, 3, 2, 1];
        let s: &[u8] = &arr;

        // Negative case
        let p = find_byte(lt(1) | eq(0), true);
        let res = parse(p, s);
        assert_eq!(res.result, None);
        assert_eq!(res.state, s);

        // Positive case with two matches. First should match
        let p = find_byte(lt(3) | lt(2), true);
        let res = parse(p, s);
        assert_eq!(res.result, Some(2));
        assert_eq!(res.state, &s[8..]);

        for i in 1_u8..arr.len() as u8 {
            let p = find_byte(lt(i + 1) | eq(i), true);
            let res = parse(p, s);
            assert_eq!(res.result, Some(i));
            assert_eq!(res.state, &s[arr.len() - (i - 1) as usize..]);
        }
    }

    #[test]
    fn greater_than() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let s: &[u8] = &arr;

        // Negative case
        let p = find_byte(gt(9) | eq(0), true);
        let res = parse(p, s);
        assert_eq!(res.result, None);
        assert_eq!(res.state, s);

        // Positive case with two matches. First should match
        let p = find_byte(gt(7) | gt(8), true);
        let res = parse(p, s);
        assert_eq!(res.result, Some(8));
        assert_eq!(res.state, &s[8..]);

        for i in 1_u8..arr.len() as u8 {
            let p = find_byte(gt(i - 1) | eq(i), true);
            let res = parse(p, s);
            assert_eq!(res.result, Some(i));
            assert_eq!(res.state, &s[i as usize..]);
        }
    }

    #[test]
    fn equals() {
        let target = 0x10_u8;
        let arr = [9, 8, 7, 6, 5, 4, 3, 2, 1];
        let s: &[u8] = &arr;
        let p = find_byte(eq(target), true);

        // Negative case
        let res = parse(p, s);
        assert_eq!(res.result, None);
        assert_eq!(res.state, s);

        // Positive case with two matches. First should match
        let arr2 = [9, 8, 7, target, target, 4, 3, 2, 1];
        let s: &[u8] = &arr2;
        let res = parse(p, s);
        assert_eq!(res.result, Some(target));
        assert_eq!(res.state, &[target, 4, 3, 2, 1]);

        // Exhaustive position
        for i in 1..s.len() {
            let p = find_byte(eq(target), true);
            let mut tmp_arr = arr;
            tmp_arr[i] = target;
            let s: &[u8] = &tmp_arr;
            let res = parse(p, s);
            assert_eq!(res.result, Some(target));
            assert_eq!(res.state, &tmp_arr[i + 1..]);
        }
    }

    #[test]
    fn not_equals() {
        let avoid = 10_u8;
        let arr = [avoid; 9];
        let s: &[u8] = &arr;
        let p = find_byte(ne(avoid), true);

        // Negative case
        let res = parse(p, s);
        assert_eq!(res.result, None);
        assert_eq!(res.state, s);

        // Positive case with two matches. First should match
        let mut arr2 = arr;
        arr2[6] = 1;
        arr2[7] = 1;
        let s: &[u8] = &arr2;
        let res = parse(p, s);
        assert_eq!(res.result, Some(1));
        assert_eq!(res.state, &[1, avoid]);

        // Exhaustive position
        for i in 1..s.len() {
            let p = find_byte(ne(avoid), true);
            let mut tmp_arr = arr;
            tmp_arr[i] = 1;
            let s: &[u8] = &tmp_arr;
            let res = parse(p, s);
            assert_eq!(res.result, Some(1));
            assert_eq!(res.state, &tmp_arr[i + 1..]);
        }
    }

    #[test]
    fn no_consume() {
        let arr: &[u8] = &[9, 8, 7, 6, 5, 4, 3, 2, 1];
        let p = find_byte(eq(7), false);
        let res = parse(p, arr);
        assert_eq!(res.result, Some(7));
        assert_eq!(res.state, &arr[2..]);
    }

    const INPUT: &[u8] = &[5, 4, 3, 2, 1, 50, 60, 70];

    fn test_finder(finder: impl ByteFinder, out: Option<u8>, consumed: usize) {
        let p = find_byte(finder, true);
        let res = parse(p, INPUT);
        assert_eq!(res.result, out);
        assert_eq!(res.state, &INPUT[consumed..]);
    }

    #[test]
    fn eq_or() {

        test_finder(eq(2) | eq(4), Some(4), 2);
        test_finder(eq(2) | eq(4), Some(4), 2);
        test_finder(eq(70) | eq(6), Some(70), 8);
    }

    #[test]
    fn combine_or() {
        test_finder(eq(2) | gt(4), Some(5), 1);
        test_finder(eq(6) | gt(55), Some(60), 7);
        test_finder(eq(2) | lt(1), Some(2), 4);
    }

    #[test]
    fn combine_and() {
        test_finder(ne(5) & ne(4), Some(3), 3);
        test_finder(lt(100) & gt(60), Some(70), 8);
        test_finder(lt(100) & gt(5), Some(50), 6);
        test_finder(lt(100) & gt(60) & ne(70), None, 0);
        test_finder(gt(50) & lt(70), Some(60), 7);
        test_finder(ne(50) & gt(20), Some(60), 7);
    }

    #[test]
    fn combine_and_or() {
        test_finder(eq(3) | (gt(50) & lt(70)), Some(3), 3);
        test_finder(gt(4) | (gt(50) & lt(70)), Some(5), 1);
        test_finder(lt(1) | (gt(50) & lt(70)), Some(60), 7);
        test_finder(lt(2) | (gt(50) & lt(70)), Some(1), 5);
    }
}
