use core::ops::{Add, Div, Mul, Sub};

use crate::{charlike::CharLike, combinators::{or, right}, core::{Parser, ParserExt}, parsers::item_if, slicelike::SliceLike};

/// Trait for types that act like numbers.
pub trait NumLike:
Add<Output = Self>
+ Sub<Output = Self>
+ Mul<Output = Self>
+ Div<Output = Self>
+ PartialOrd
+ Copy {
    const MIN: Self;
    const MAX: Self;
    const SIZE: usize;
    fn cast_u8(n: u8) -> Self;
}

/// Trait for types that act like floating point numbers.
pub trait FloatLike: Add<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Copy {
    const ONE: Self;
    const MINUS_ONE: Self;
    fn cast_usize(n: usize) -> Self;
    fn cast_isize(n: isize) -> Self;
}

macro_rules! impl_NumLike {
    ($($type:tt),*) => {
        $(
            impl NumLike for $type {
                const MIN: $type = $type::MIN;
                const MAX: $type = $type::MAX;
                const SIZE: usize = core::mem::size_of::<$type>();

                #[inline(always)]
                fn cast_u8(n: u8) -> Self {
                    n as $type
                }
            }
        )*
    }
}

macro_rules! impl_FloatLike {
    ($($type:tt),*) => {
        $(
            impl FloatLike for $type {
                const ONE: Self = 1.0;
                const MINUS_ONE: Self = -1.0;
                #[inline(always)]
                fn cast_usize(n: usize) -> Self {
                    n as $type
                }

                #[inline(always)]
                fn cast_isize(n: isize) -> Self {
                    n as $type
                }
            }
        )*
    }
}


impl_NumLike!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_FloatLike!(f32, f64);

#[inline(always)]
fn integer_internal<const CHECKED: bool, const NEG: bool, const DEC_DIVISOR: bool,
                    O: NumLike,
                    A: CharLike,
                    I: SliceLike<RefItem = A>,
                    S>() -> impl Parser<I, (O, usize, bool), S> {
    create_parser!(s, {
        let mut idx: I::Idx = Default::default();
        let mut acc = O::cast_u8(0);
        let mut dec_divisor = 1;

        // The number 10 is guaranteed to fit into all our `NumLike` types
        let ten = O::cast_u8(10);
        let mut iter = s.input.slice_iter();
        let mut consume = |digit: u32, is_negative: bool| -> Option<()> {
            // Digits are between 0 and 9, so they always fit in all types
            let digit = O::cast_u8(digit as u8);

            if CHECKED && acc > (O::MAX / ten) {
                return None
            }
            acc = acc * ten;

            if is_negative {
                if CHECKED && acc < O::MIN + digit {
                    return None
                }
                acc = acc - digit;
            } else {
                if CHECKED && acc > O::MAX - digit {
                    return None
                }
                acc = acc + digit;
            }
            idx += true.into();
            if DEC_DIVISOR {
                dec_divisor *= 10;
            }

            Some(())
        };

        let is_negative = if NEG {
            let c = iter.next()?;
            if c.as_char() == '-' {
                true
            } else {
                // We don't care about checking the result here, since a single digit can never fail.
                consume(c.as_char().to_digit(10)?, false);
                false
            }
        } else {
            false
        };

        for digit in iter.map_while(|d| d.as_char().to_digit(10)) {
            consume(digit, is_negative)?;
        }

        if idx == Default::default() {
            None
        } else {
            s.input = s.input.slice_from(idx + is_negative.into());
            Some((acc, dec_divisor, is_negative))
        }
    })
}

/// Parse an unsigned integer. The type of the integer will be inferred from the context.
#[inline]
pub fn integer<O: NumLike,
               A: CharLike,
               I: SliceLike<RefItem = A>,
               S>() -> impl Parser<I, O, S> {
    integer_internal::<false, false, false,_,_,_,_>().map(|(n, _, _)| n)
}

/// Parse an unsigned integer. The type of the integer will be inferred from the context.
/// This parser will fail if the result does not fit in the inferred integer type.
#[inline]
pub fn integer_checked<O: NumLike,
                       A: CharLike,
                       I: SliceLike<RefItem = A>,
                       S>() -> impl Parser<I, O, S> {
    integer_internal::<true, false, false,_,_,_,_>().map(|(n, _, _)| n)
}

/// Parse an signed integer. The type of the integer will be inferred from the context.
#[inline]
pub fn integer_signed<O: NumLike,
                      A: CharLike,
                      I: SliceLike<RefItem = A>,
                      S>() -> impl Parser<I, O, S> {
    integer_internal::<false, true, false,_,_,_,_>().map(|(n, _, _)| n)
}

/// Parse an signed integer. The type of the integer will be inferred from the context.
/// This parser will fail if the result does not fit in the inferred integer type.
#[inline]
pub fn integer_signed_checked<O: NumLike,
                              A: CharLike,
                              I: SliceLike<RefItem = A>,
                              S>() -> impl Parser<I, O, S> {
    integer_internal::<true, true, false,_,_,_,_>().map(|(n, _, _)| n)
}

#[inline(always)]
fn float_internal<const CHECKED: bool,
                  O: FloatLike,
                  A: CharLike,
                  I: SliceLike<RefItem = A>,
                  S>() -> impl Parser<I, O, S> {
    // First parse a possibly negative signed integer
    integer_internal::<CHECKED, true, false,_,_,_,_>().bind(|(n, _, is_neg)| {
        // Then parse a period followed by an unsigned integer.
        let dec = right(item_if(|c: I::RefItem| c.as_char() == '.'),
                                              integer_internal::<CHECKED,false,true,_,_,_,_>())
            .map(move |(dec, div, _)|
                O::cast_isize(n) + if is_neg {O::MINUS_ONE} else {O::ONE} * O::cast_usize(dec) / O::cast_usize(div));
        or(dec, pure!(O::cast_isize(n)))
    })
}

/// Parse a floating point number. The type of the number will be inferred from the context.
/// This parser is incomplete, in that it will attempt to parse the float as
/// `isize.usize`, and if the parsed number does not fit within those types, it will panic.
#[inline]
pub fn float<O: FloatLike, A: CharLike, I: SliceLike<RefItem = A>, S>() -> impl Parser<I, O, S> {
    float_internal::<false,_,_,_,_>()
}

/// Parse a floating point number. The type of the number will be inferred from the context.
/// This parser is incomplete, in that it will attempt to parse the float as
/// `isize.usize`, and if the parsed number does not fit within those types, it will fail.
#[inline]
pub fn float_checked<O: FloatLike,
                     A: CharLike,
                     I: SliceLike<RefItem = A>,
                     S>() -> impl Parser<I, O, S> {
    float_internal::<true,_,_,_,_>()
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, number::{integer, integer_checked, float, integer_signed, integer_signed_checked}};

    #[test]
    fn unsigned_integer() {
        assert_eq!(0, parse(integer(), "0").result.unwrap());
        assert_eq!(127, parse(integer(), "127").result.unwrap());
        assert_eq!(255, parse(integer(), "255").result.unwrap());

        assert!((parse(integer(), "-1").result as Option<u8>).is_none());
        assert!((parse(integer_checked(), "256").result as Option<u8>).is_none());
    }

    #[test]
    fn signed_integer() {
        assert_eq!(0, parse(integer_signed(), "0").result.unwrap());
        assert_eq!(127, parse(integer_signed(), "127").result.unwrap());
        assert_eq!(-1, parse(integer_signed(), "-1").result.unwrap());
        assert_eq!(-128, parse(integer_signed(), "-128").result.unwrap());

        assert_eq!(128u8, parse(integer_signed(), "128").result.unwrap());

        assert!((parse(integer_signed_checked(), "-129").result as Option<u8>).is_none());
        assert!((parse(integer_signed_checked(), "128").result as Option<i8>).is_none());
    }

    #[test]
    fn float_test() {
        assert_eq!(0f32, parse(float(), "0").result.unwrap());
        assert_eq!(100000000f32, parse(float(), "100000000").result.unwrap());
        assert_eq!(-100000000f32, parse(float(), "-100000000").result.unwrap());
        assert_eq!(13.37f32, parse(float(), "13.37").result.unwrap());
        assert_eq!(-13.37f32, parse(float(), "-13.37").result.unwrap());
        assert_eq!(13.07f32, parse(float(), "13.07").result.unwrap());
        assert_eq!(-13.07f32, parse(float(), "-13.07").result.unwrap());
        assert_eq!(1.123f32, parse(float(), "1.123").result.unwrap());
        assert_eq!(0.001f32, parse(float(), "0.001").result.unwrap());
        assert_eq!(-0.001f32, parse(float(), "-0.001").result.unwrap());
    }
}