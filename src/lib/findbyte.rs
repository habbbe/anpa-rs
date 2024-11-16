/// One unit of "work". In this case `usize` will process 8 bytes
/// at a time on a 64-bit CPU (or 4 bytes on 32-bit).
pub type Work = usize;

/// Trait for types that can be used for finding a byte in a wider
/// integer type.
pub trait ByteFinder {
    /// Produce an intermediate format used for confirming existence
    /// of byte. The result will be applied to `& 0x808080...` as a final
    /// step.
    fn intermediate(&self, haystack: Work) -> Work;

    /// The "traditional" comparison. Used when a full work unit cannot
    /// be obtained.
    fn slow_cmp(&self, other: u8) -> bool;
}

pub const LOW_BITS: Work = Work::MAX / 255;
pub const HIGH_BITS: Work = LOW_BITS << 7;

/// A wrapper used for finding a byte that is smaller than
/// the wrappee.
pub struct LtByte {
    pub b: u8
}

impl LtByte {
    #[inline(always)]
    pub fn new(b: u8) -> LtByte {
        LtByte { b }
    }
}

/// A wrapper used for finding a byte that is greater than
/// the wrappee.
pub struct GtByte {
    pub b: u8
}

impl GtByte {
    #[inline(always)]
    pub fn new(b: u8) -> GtByte {
        GtByte { b }
    }
}

/// A wrapper used for finding a byte that is not equal to
/// the wrappee.
pub struct NeByte {
    pub b: u8
}

impl NeByte {
    #[inline(always)]
    pub fn new(b: u8) -> NeByte {
        NeByte { b }
    }
}

impl ByteFinder for u8 {
    #[inline]
    fn intermediate(&self, haystack: Work) -> Work {
        let to_find = haystack ^ (*self as Work * LOW_BITS);
        to_find.wrapping_sub(LOW_BITS) & !to_find
    }

    #[inline(always)]
    fn slow_cmp(&self, other: u8) -> bool {
        *self == other
    }
}

impl ByteFinder for LtByte {
    #[inline]
    fn intermediate(&self, haystack: Work) -> Work {
        haystack.wrapping_sub(LOW_BITS * self.b as Work) & !haystack
    }

    #[inline(always)]
    fn slow_cmp(&self, other: u8) -> bool {
        other < self.b
    }
}

impl ByteFinder for GtByte {
    #[inline]
    fn intermediate(&self, haystack: Work) -> Work {
        let mask = LOW_BITS * self.b as Work;
        mask.wrapping_sub(haystack) & !mask
    }

    #[inline(always)]
    fn slow_cmp(&self, other: u8) -> bool {
        other > self.b
    }
}

impl ByteFinder for NeByte {
    #[inline]
    fn intermediate(&self, haystack: Work) -> Work {
        self.b.intermediate(haystack) ^ HIGH_BITS
    }

    #[inline(always)]
    fn slow_cmp(&self, other: u8) -> bool {
        other != self.b
    }
}

/// Find a single byte in an input that can be represented as a
/// contiguous area of bytes. It will process multiple bytes at
/// a time, more specifically `usize::BITS / 8` bytes.
///
/// When searching for individual bytes, this is likely faster than using
/// [`until`](crate::parsers::until), or using
/// [`item_if`](crate::parsers::item_if) together with
/// [`many`](crate::combinators::many)
///
/// The macro is variadic, and every provided argument will be sought
/// individually. The first matching argument will be returned as the
/// result.
///
/// Available argment types are:
/// - `u8`: Search for a byte via equality
/// - [`NeByte`]: Search for a byte via inequality, i.e. `NeByte::new(10)``
///             will search for a byte not equal to 10.
/// - [`LtByte`]: Search for a byte via "less than", i.e. LtByte::new(10)
///             will search for a byte smaller than 10.
/// - [`GtByte`]: Search for a byte via "greater than", i.e. GtByte::new(10)
///             will search for a byte greater than 10.
///
/// ### Consuming
/// Consumes all items before the match, and the match itself.
///
/// ### Example:
/// ```
/// use anpa::core::*;
/// use anpa::find_byte;
/// use anpa::findbyte::*;
///
/// // Find ascii `"`, `\` or control character.
/// let p = find_byte!(b'"', b'\\', LtByte::new(0x20));
///
/// let input1 = "abcd\"";
/// let input2 = "ab\\cd";
/// let input3 = "a\nbcd";
///
/// assert_eq!(parse(p, input1).result, Some(b'"'));
/// assert_eq!(parse(p, input2).result, Some(b'\\'));
/// assert_eq!(parse(p, input3).result, Some(b'\n'));
/// ```
#[macro_export]
macro_rules! find_byte {
    (impl $include:literal, $($mul:expr),+ $(,)?) => {
        $crate::create_parser!(s, {
            let mut pos = 0;
            let res;
            {
                let bytes = $crate::slicelike::ContiguousBytes::to_u8_slice(&s.input);

                let mut chunks = bytes.chunks_exact($crate::findbyte::Work::BITS as usize / 8);

                'outer: loop {
                    while let Some(chunk) = chunks.next() {
                        let val = $crate::findbyte::Work::from_le_bytes(
                            core::convert::TryInto::try_into(chunk).unwrap());
                        let present = ($( $crate::findbyte::ByteFinder::intermediate(&$mul, val)) |*) & $crate::findbyte::HIGH_BITS;

                        if present != 0 {
                            pos = pos + (present.trailing_zeros() / u8::BITS) as usize;
                            break 'outer
                        }

                        pos += $crate::findbyte::Work::BITS as usize / 8;
                    }

                    pos = chunks.remainder().iter()
                        .position(|x| $($crate::findbyte::ByteFinder::slow_cmp(&$mul, *x)) ||*)? + pos;

                    break
                }

                res = bytes[pos];
            }

            if !$include {
                pos += 1;
            }

            s.input = $crate::slicelike::SliceLike::slice_from(s.input,
                $crate::slicelike::SliceLike::slice_idx_from_offset(s.input, pos));

            return Some(res)
        })
    };

    ($($mul:expr),+ $(,)?) => {
        find_byte!(impl false, $($mul),*)
    };
}

/// A version of [`find_byte!`] that leaves the found byte
/// in the input.
///
/// ### Consuming
/// Consumes all items before the match, but not the match itself.
///
/// See the aforemented for examples.
#[macro_export]
macro_rules! find_byte_keep {
    ($($mul:expr),+ $(,)?) => {
        $crate::find_byte!(impl true, $($mul),*)
    };
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, findbyte::{LtByte, GtByte, NeByte}};

    #[test]
    fn less_than() {
        let arr = [9, 8, 7, 6, 5, 4, 3, 2, 1];
        let s: &[u8] = &arr;

        // Negative case
        let p = find_byte!(LtByte::new(1), 0);
        let res = parse(p, s);
        assert_eq!(res.result, None);
        assert_eq!(res.state, s);

        // Positive case with two matches. First should match
        let p = find_byte!(LtByte::new(3), LtByte::new(2));
        let res = parse(p, s);
        assert_eq!(res.result, Some(2));
        assert_eq!(res.state, &s[8..]);

        for i in 1_u8..arr.len() as u8 {
            let p = find_byte!(LtByte::new(i + 1), i);
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
        let p = find_byte!(GtByte::new(9), 0);
        let res = parse(p, s);
        assert_eq!(res.result, None);
        assert_eq!(res.state, s);

        // Positive case with two matches. First should match
        let p = find_byte!(GtByte::new(7), GtByte::new(8));
        let res = parse(p, s);
        assert_eq!(res.result, Some(8));
        assert_eq!(res.state, &s[8..]);

        for i in 1_u8..arr.len() as u8 {
            let p = find_byte!(GtByte::new(i - 1), i);
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
        let p = find_byte!(target);

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
            let p = find_byte!(target);
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
        let p = find_byte!(NeByte::new(avoid));

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
            let p = find_byte!(NeByte::new(avoid));
            let mut tmp_arr = arr;
            tmp_arr[i] = 1;
            let s: &[u8] = &tmp_arr;
            let res = parse(p, s);
            assert_eq!(res.result, Some(1));
            assert_eq!(res.state, &tmp_arr[i + 1..]);
        }
    }

    #[test]
    fn byte_slice() {
        let s: &[u8] = &[5, 4, 3, 2, 1, 1, 1, 1];

        let p = find_byte!(2, 4);
        let res = parse(p, s);
        assert_eq!(res.result, Some(4));
        assert_eq!(res.state, &[3, 2, 1, 1, 1, 1]);

        let p = find_byte_keep!(2, 4);
        let res = parse(p, s);
        assert_eq!(res.result, Some(4));
        assert_eq!(res.state, &[4, 3, 2, 1, 1, 1, 1]);

        let p = find_byte!(3, LtByte::new(5));
        let res = parse(p, s);
        assert_eq!(res.result, Some(4));
        assert_eq!(res.state, &[3, 2, 1, 1, 1, 1]);
    }
}
