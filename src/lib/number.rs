use std::ops::{Add, Sub, Mul, Div};

use crate::{slicelike::SliceLike, core::{Parser, ParserExt, AnpaState}, combinators::{right, or}, parsers::item_if, asciilike::AsciiLike};

pub trait NumLike:
Add<Output = Self>
+ Sub<Output = Self>
+ Mul<Output = Self>
+ Div<Output = Self>
+ PartialOrd
+ Copy {
    const MIN: Self;
    const MAX: Self;
    fn cast_u8(n: u8) -> Self;
}

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
                    A: AsciiLike,
                    I: SliceLike<RefItem = A>,
                    S>() -> impl Parser<I, (O, usize), S> {
    create_parser!(s, {
        let mut idx = 0;
        let mut acc = O::cast_u8(0);
        let mut dec_divisor = 1;

        // The number 10 is guaranteed to fit into all our `NumLike` types
        let ten = O::cast_u8(10);
        let mut iter = s.input.slice_iter();
        let mut consume = |digit: u8, is_negative: bool, checked: bool| -> Option<()> {
            // Digits are between 0 and 9, so they always fit in all types
            let digit = O::cast_u8(digit);

            if checked {
                if acc > (O::MAX / ten) {
                    return None
                }
            }
            acc = acc * ten;

            if is_negative {
                if checked {
                    if acc < O::MIN + digit {
                        return None
                    }
                }
                acc = acc - digit;
            } else {
                if checked {
                    if acc > O::MAX - digit {
                        return None
                    }
                }
                acc = acc + digit;
            }
            idx += 1;
            if DEC_DIVISOR {
                dec_divisor *= 10;
            }

            Some(())
        };

        let is_negative = if NEG {
            let c = iter.next()?;
            if c.equal(A::MINUS) {
                true
            } else {
                // We don't care about checking the result here, since a single digit can never fail.
                consume(c.as_digit()?, false, false);
                false
            }
        } else {
            false
        };

        for digit in iter.map_while(A::as_digit) {
            consume(digit, is_negative, CHECKED)?;
        }

        if idx == 0 {
            None
        } else {
            s.input = s.input.slice_from(idx + (is_negative as usize));
            Some((acc, dec_divisor))
        }
    })
}

#[inline]
pub fn integer<O: NumLike,
               A: AsciiLike,
               I: SliceLike<RefItem = A>,
               S>() -> impl Parser<I, O, S> {
    integer_internal::<false, false, false,_,_,_,_>().map(|(n, _)| n)
}

#[inline]
pub fn integer_checked<O: NumLike,
                       A: AsciiLike,
                       I: SliceLike<RefItem = A>,
                       S>() -> impl Parser<I, O, S> {
    integer_internal::<true, false, false,_,_,_,_>().map(|(n, _)| n)
}

#[inline]
pub fn integer_signed<O: NumLike,
                      A: AsciiLike,
                      I: SliceLike<RefItem = A>,
                      S>() -> impl Parser<I, O, S> {
    integer_internal::<false, true, false,_,_,_,_>().map(|(n, _)| n)
}

#[inline]
pub fn integer_signed_checked<O: NumLike,
                              A: AsciiLike,
                              I: SliceLike<RefItem = A>,
                              S>() -> impl Parser<I, O, S> {
    integer_internal::<true, true, false,_,_,_,_>().map(|(n, _)| n)
}

#[inline(always)]
fn float_internal<const CHECKED: bool,
                  O: FloatLike,
                  A: AsciiLike,
                  I: SliceLike<RefItem = A>,
                  S>() -> impl Parser<I, O, S> {
    // First parse a possibly negative signed integer
    integer_internal::<CHECKED, true, true,_,_,_,_>().bind(|(n, div): (isize, _)| {
        // Then parse a period followed by an unsigned integer.
        let dec = right(item_if(|c: I::RefItem| c.equal(A::PERIOD)),
                                              integer_internal::<CHECKED,false,false,_,_,_,_>())
            .map(move |(dec, _): (usize,_)|
                O::cast_isize(n) + if n.is_negative() {O::MINUS_ONE} else {O::ONE} * O::cast_usize(dec) / O::cast_usize(div));
        or(dec, pure!(O::cast_isize(n)))
    })
}

#[inline]
pub fn float<O: FloatLike, A: AsciiLike, I: SliceLike<RefItem = A>, S>() -> impl Parser<I, O, S> {
    float_internal::<false,_,_,_,_>()
}

#[inline]
pub fn float_checked<O: FloatLike,
                     A: AsciiLike,
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
    }
}