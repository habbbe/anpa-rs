use core::ops::{Add, Div, Mul, Sub};

use crate::{charlike::CharLike, combinators::{bind, map, or, right}, core::{Parser, ParserExt}, parsers::item_if, slicelike::SliceLike};

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
    const ZERO: Self;
    const TEN: Self;
    const SIZE: usize;
    fn cast_u8(n: u8) -> Self;
    fn checked_add(self, n: Self) -> Option<Self>;
    fn checked_sub(self, n: Self) -> Option<Self>;
    fn checked_mul(self, n: Self) -> Option<Self>;
}

/// Trait for types that act like floating point numbers.
pub trait FloatLike: Add<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Copy {
    const ZERO: Self;
    const ONE: Self;
    const TEN: Self;
    const MINUS_ONE: Self;
    fn cast_usize(n: usize) -> Self;
    fn cast_isize(n: isize) -> Self;
    fn pow_i(self, exp: i32) -> Self;
}

macro_rules! impl_NumLike {
    ($($type:tt),*) => {
        $(
            impl NumLike for $type {
                const MIN: $type = $type::MIN;
                const MAX: $type = $type::MAX;
                const ZERO: $type = 0;
                const TEN: $type = 10;
                const SIZE: usize = core::mem::size_of::<$type>();

                #[inline(always)]
                fn cast_u8(n: u8) -> Self {
                    n as $type
                }

                #[inline(always)]
                fn checked_add(self, n: Self) -> Option<Self> {
                    self.checked_add(n)
                }

                #[inline(always)]
                fn checked_sub(self, n: Self) -> Option<Self> {
                    self.checked_sub(n)
                }

                #[inline(always)]
                fn checked_mul(self, n: Self) -> Option<Self> {
                    self.checked_mul(n)
                }
            }
        )*
    }
}

macro_rules! impl_FloatLike {
    ($($type:tt),*) => {
        $(
            impl FloatLike for $type {
                const ZERO: Self = 0.0;
                const ONE: Self = 1.0;
                const TEN: Self = 10.0;
                const MINUS_ONE: Self = -1.0;
                #[inline(always)]
                fn cast_usize(n: usize) -> Self {
                    n as $type
                }

                #[inline(always)]
                fn cast_isize(n: isize) -> Self {
                    n as $type
                }

                #[inline(always)]
                fn pow_i(self, exp: i32) -> Self {
                    self.powi(exp)
                }
            }
        )*
    }
}


impl_NumLike!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_FloatLike!(f32, f64);

pub const fn integer_internal<const CHECKED: bool,
                              const SIGNED: bool,
                              const LEADING_PLUS: bool,
                              const LEADING_ZEROS: bool,
                              const DEC_DIVISOR: bool,
                              O: NumLike,
                              A: CharLike,
                              I: SliceLike<RefItem = A>,
                              S>() -> impl Parser<I, (O, usize, bool), S> {
    create_parser!(s, {
        let mut idx = I::Idx::default();
        let mut acc = O::ZERO;
        let mut dec_divisor = 1;

        let mut iter = s.input.slice_iter();
        let mut consume = |digit: u32, is_negative: bool| -> Option<()> {
            // Digits are between 0 and 9, so they always fit in all types
            let digit = O::cast_u8(digit as u8);

            if !LEADING_ZEROS {
                if acc == O::ZERO && idx != I::Idx::default() {
                    return None
                }
            }

            if CHECKED {
                acc = acc.checked_mul(O::TEN)?;
            } else {
                acc = acc * O::TEN;
            }

            if is_negative {
                if CHECKED {
                    acc = acc.checked_sub(digit)?;
                } else {
                    acc = acc - digit;
                }
            } else if CHECKED {
                acc = acc.checked_add(digit)?;
            } else {
                acc = acc + digit;
            }

            idx += true.into();

            if DEC_DIVISOR {
                if CHECKED {
                    dec_divisor = dec_divisor.checked_mul(10)?;
                } else {
                    dec_divisor *= 10;
                }
            }

            Some(())
        };

        let leading = if SIGNED || LEADING_PLUS {
            let c = iter.next()?.as_char();

            if SIGNED && c == '-' {
                Some(true)
            } else if LEADING_PLUS && c == '+' {
                Some(false)
            } else {
                // We don't care about checking the result here, since a single digit can never fail.
                consume(c.to_digit(10)?, false);
                None
            }
        } else {
            None
        };

        let is_negative = leading.unwrap_or(false);

        for digit in iter.map_while(|d| d.as_char().to_digit(10)) {
            consume(digit, is_negative)?;
        }

        (idx != Default::default()).then(|| {
            s.input = s.input.slice_from(idx + leading.is_some().into());
            (acc, dec_divisor, is_negative)
        })
    })
}

/// Configuration for integer parsing. The instance functions can be used to
/// change the behavior of the parse.
pub struct IntConfig<const CHECKED: bool = true,
                     const SIGNED: bool = false,
                     const LEADING_PLUS: bool = false,
                     const LEADING_ZEROS: bool = true>;

impl IntConfig {
    pub const fn new() -> Self {
        IntConfig
    }
}

impl<const CHECKED: bool,
     const SIGNED: bool,
     const LEADING_PLUS: bool,
     const LEADING_ZEROS: bool>
     IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS> {
    pub const fn unchecked(self) -> IntConfig<false, SIGNED, LEADING_PLUS, LEADING_ZEROS> {
        IntConfig
    }

    pub const fn signed(self) -> IntConfig<CHECKED, true, LEADING_PLUS, LEADING_ZEROS> {
        IntConfig
    }

    pub const fn no_leading_zero(self) -> IntConfig<CHECKED, SIGNED, LEADING_PLUS, false> {
        IntConfig
    }

    pub const fn leading_plus(self) -> IntConfig<CHECKED, SIGNED, true, LEADING_ZEROS> {
        IntConfig
    }
}

/// Parse an unsigned integer. The type of the integer will be inferred from the context.
/// General verison taking a config object. See [`IntConfig`] for more information.
///
/// ### Arguments
/// * `_config` - the configuration object
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::{integer_custom, IntConfig};
///
/// let parse_plus = integer_custom(IntConfig::new().leading_plus());
/// let parse_no_plus = integer_custom(IntConfig::new());
/// let input1 = "+10";
/// let input2 = "10";
///
/// assert_eq!(Some(10), parse(parse_plus, input1).result);
/// assert_eq!(Some(10), parse(parse_plus, input2).result);
///
/// assert_eq!(None, parse(parse_no_plus, input1).result);
/// assert_eq!(Some(10), parse(parse_no_plus, input2).result);
/// ```
#[inline]
pub const fn integer_custom<const CHECKED: bool,
                            const SIGNED: bool,
                            const LEADING_PLUS: bool,
                            const LEADING_ZEROS: bool,
                            O: NumLike,
                            A: CharLike,
                            I: SliceLike<RefItem = A>,
                            S>(_config: IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS>) -> impl Parser<I, O, S> {
    map(integer_internal::<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, false,_,_,_,_>(), |(n,_,_)| n)
}

/// Parse an unsigned integer. The type of the integer will be inferred from the context.
/// This parser will fail if the result does not fit in the inferred integer type.
#[inline]
pub const fn integer<O: NumLike,
                     A: CharLike,
                     I: SliceLike<RefItem = A>,
                     S>() -> impl Parser<I, O, S> {
    integer_custom(IntConfig::new())
}

/// Parse an signed integer. The type of the integer will be inferred from the context.
/// This parser will fail if the result does not fit in the inferred integer type.
#[inline]
pub const fn integer_signed<O: NumLike,
                            A: CharLike,
                            I: SliceLike<RefItem = A>,
                            S>() -> impl Parser<I, O, S> {
    integer_custom(IntConfig::new().signed())
}

/// Configuration for float parsing. The instance functions can be used to
/// change the behavior of the parse.
pub struct FloatConfig<const CHECKED: bool = true,
                       const SIGNED: bool = true,
                       const SCI: bool = false,
                       const LEADING_PLUS: bool = false,
                       const LEADING_ZEROS_INT: bool = true,
                       const LEADING_ZEROS_EXP: bool = true,
                       const DECIMAL_COMMA: bool = false>;

impl FloatConfig {
    pub const fn new() -> Self {
        FloatConfig
    }
}

impl<const CHECKED: bool,
     const SIGNED: bool,
     const SCI: bool,
     const LEADING_PLUS: bool,
     const LEADING_ZERO_INT: bool,
     const LEADING_ZERO_EXP: bool,
     const DECIMAL_COMMA: bool>
     FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA> {

    pub const fn unchecked(self) -> FloatConfig::<false, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA> {
        FloatConfig
    }

    pub const fn unsigned(self) -> FloatConfig::<CHECKED, false, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA> {
        FloatConfig
    }

    pub const fn scientific(self) -> FloatConfig::<CHECKED, SIGNED, true, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA> {
        FloatConfig
    }

    pub const fn leading_plus(self) -> FloatConfig::<CHECKED, SIGNED, SCI, true, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA> {
        FloatConfig
    }

    pub const fn no_leading_zero_int(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, false, LEADING_ZERO_EXP, DECIMAL_COMMA> {
        FloatConfig
    }

    pub const fn no_leading_zero_exp(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, false, DECIMAL_COMMA> {
        FloatConfig
    }

    pub const fn decimal_comma(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, true> {
        FloatConfig
    }
}

/// Parse a float. The type of the float will be inferred from the context.
/// General verison taking a config object. See [`FloatConfig`] for more information.
///
/// ### Arguments
/// * `_config` - the configuration object
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::{float_custom, FloatConfig};
///
/// let parse_sci = float_custom(FloatConfig::new().scientific());
/// let parse_no_sci = float_custom(FloatConfig::new());
/// let input1 = "1.02e30";
/// let input2 = "10.2";
///
/// assert_eq!(Some(1.02e30), parse(parse_sci, input1).result);
/// assert_eq!(Some(10.2), parse(parse_sci, input2).result);
///
/// assert_eq!(Some(1.02), parse(parse_no_sci, input1).result);
/// assert_eq!(Some(10.2), parse(parse_no_sci, input2).result);
/// ```
#[inline]
pub const fn float_custom<const CHECKED: bool,
                          const SIGNED: bool,
                          const SCI: bool,
                          const LEADING_PLUS: bool,
                          const LEADING_ZERO_INT: bool,
                          const LEADING_ZERO_EXP: bool,
                          const DECIMAL_COMMA: bool,
                          O: FloatLike,
                          A: CharLike,
                          I: SliceLike<RefItem = A>,
                          S>(_config: FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA>)
                          -> impl Parser<I, O, S> {

    // First parse a possibly negative signed integer
    bind(integer_internal::<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZERO_INT, false,_,_,_,_>(), |(n, _, is_neg)| {
        // Then parse a period followed by an unsigned integer.
        let int = O::cast_isize(n);
        let dec = right(item_if(|c: I::RefItem| c.as_char() ==  if DECIMAL_COMMA {','} else {'.'}),
                  integer_internal::<CHECKED, false, false, true, true,_,_,_,_>())
            .map(move |(dec, div, _)|
                int + if is_neg {O::MINUS_ONE} else {O::ONE} * O::cast_usize(dec) / O::cast_usize(div));

        let pre_exp_parser = or(dec, pure!(int));

        choose_pure!(SCI;
            true => bind(pre_exp_parser, |pre_exp| {
                let exp = right(item_if(|c: I::RefItem| matches!(c.as_char(), 'e' | 'E')),
                                integer_custom(IntConfig::<CHECKED, true, true, LEADING_ZERO_EXP>))
                    .map(move |exp| pre_exp * O::TEN.pow_i(exp));
                or(exp, pure!(pre_exp))
            }),
            false => pre_exp_parser
        )
    })
}

/// Parse a floating point number. The type of the number will be inferred from the context.
/// This parser will attempt to parse the float as `isize.usize`, and if the parsed number
/// does not fit within those types, it will fail.
#[inline]
pub const fn float<O: FloatLike,
                   A: CharLike,
                   I: SliceLike<RefItem = A>,
                   S>() -> impl Parser<I, O, S> {
    float_custom(FloatConfig::new())
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, number::{float, integer, integer_signed}};

    #[test]
    fn unsigned_integer() {
        assert_eq!(0, parse(integer(), "0").result.unwrap());
        assert_eq!(127, parse(integer(), "127").result.unwrap());
        assert_eq!(255, parse(integer(), "255").result.unwrap());

        assert!((parse(integer(), "-1").result as Option<u8>).is_none());
        assert!((parse(integer(), "256").result as Option<u8>).is_none());
    }

    #[test]
    fn signed_integer() {
        assert_eq!(0, parse(integer_signed(), "0").result.unwrap());
        assert_eq!(127, parse(integer_signed(), "127").result.unwrap());
        assert_eq!(-1, parse(integer_signed(), "-1").result.unwrap());
        assert_eq!(-128, parse(integer_signed(), "-128").result.unwrap());

        assert_eq!(128u8, parse(integer_signed(), "128").result.unwrap());

        assert!((parse(integer_signed(), "-129").result as Option<u8>).is_none());
        assert!((parse(integer_signed(), "128").result as Option<i8>).is_none());
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
