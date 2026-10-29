use core::{ops::{Add, Div, Mul, Sub}, str::FromStr};

use crate::{charlike::CharLike, combinators::{attempt, bind, from_str, get_parsed, map, succeed}, core::{Parser, ParserExt}, parsers::item_if, slicelike::SliceLike};


/// Struct representing a float by its parts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatParts<F: FloatLike> {
    pub negative: bool,
    pub mantissa: F::IntRep,
    pub exp: i32,
}

impl<F: FloatLike> FloatParts<F> {
    pub fn to_float_fast(&self) -> F {
        let pre_sign = if self.mantissa == F::IntRep::ZERO {
            F::ZERO
        } else {
            let mantissa_float = F::cast_int_rep(self.mantissa);
            if self.exp < 0 {
                // Using division for negative exponents seem to
                // incur less rounding errors.
                mantissa_float / F::TEN.pow_i(-self.exp)
            } else {
                mantissa_float * F::TEN.pow_i(self.exp)
            }
        };

        if self.negative {
            pre_sign * F::MINUS_ONE
        } else {
            pre_sign
        }
    }
}

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
    const ONE: Self;
    const TEN: Self;
    const SIZE: usize;
    const SIGNED: bool;
    fn cast_u8(n: u8) -> Self;
    fn checked_add(self, n: Self) -> Option<Self>;
    fn checked_sub(self, n: Self) -> Option<Self>;
    fn checked_mul(self, n: Self) -> Option<Self>;
    fn from_le_bytes(bytes: &[u8]) -> Option<(Self, &[u8])>;
    fn from_be_bytes(bytes: &[u8]) -> Option<(Self, &[u8])>;
    fn from_ne_bytes(bytes: &[u8]) -> Option<(Self, &[u8])>;
    fn max_safe_digits<const BASE: u8>() -> i32;
}

/// Trait for types that act like floating point numbers.
pub trait FloatLike: Add<Output = Self> + Mul<Output = Self> + Div<Output = Self> + PartialEq + Copy {
    const ZERO: Self;
    const MINUS_ZERO: Self;
    const ONE: Self;
    const TEN: Self;
    const MINUS_ONE: Self;
    type IntRep: NumLike;
    fn cast_usize(n: usize) -> Self;
    fn cast_isize(n: isize) -> Self;
    fn cast_int_rep(n: Self::IntRep) -> Self;
    fn pow_i(self, exp: i32) -> Self;
}

macro_rules! impl_NumLike {
    ($($type:tt),*) => {
        $(
            impl NumLike for $type {
                const MIN: $type = $type::MIN;
                const MAX: $type = $type::MAX;
                const ZERO: $type = 0;
                const ONE: $type = 1;
                const TEN: $type = 10;
                const SIZE: usize = core::mem::size_of::<$type>();

                #[allow(unused_comparisons)]
                const SIGNED: bool = 0 > $type::MIN;

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

                fn from_le_bytes(bytes: &[u8]) -> Option<(Self, &[u8])> {
                    bytes.split_first_chunk::<{Self::SIZE}>()
                        .map(|(chunk, rest)| ($type::from_le_bytes(*chunk), rest))
                }

                fn from_be_bytes(bytes: &[u8]) -> Option<(Self, &[u8])> {
                    bytes.split_first_chunk::<{Self::SIZE}>()
                        .map(|(chunk, rest)| ($type::from_be_bytes(*chunk), rest))
                }

                fn from_ne_bytes(bytes: &[u8]) -> Option<(Self, &[u8])> {
                    bytes.split_first_chunk::<{Self::SIZE}>()
                        .map(|(chunk, rest)| ($type::from_ne_bytes(*chunk), rest))
                }

                /// Calculate the safe maximum number of digits that can fit in this integer.
                /// That is, until we reach this number, checked arithmetic is not needed.
                fn max_safe_digits<const BASE: u8>() -> i32 {
                    const {
                        assert!(BASE >= 2, "Base must be 2 or greater");

                        let n = (BASE - 1) as $type;
                        let mut digits = 1;
                        let mut acc = n;
                        loop {
                            let Some(mul) = acc.checked_mul(BASE as $type) else {
                                break;
                            };

                            let Some(next) = mul.checked_add(n) else {
                                break;
                            };

                            digits += 1;
                            acc = next;
                        }
                        digits
                    }
                }
            }
        )*
    }
}

macro_rules! impl_FloatLike {
    ($(($type:tt, $int:ty)),*) => {
        $(
            impl FloatLike for $type {
                const ZERO: Self = 0.0;
                const MINUS_ZERO: Self = -0.0;
                const ONE: Self = 1.0;
                const TEN: Self = 10.0;
                const MINUS_ONE: Self = -1.0;
                type IntRep = $int;
                #[inline(always)]
                fn cast_usize(n: usize) -> Self {
                    n as Self
                }

                #[inline(always)]
                fn cast_isize(n: isize) -> Self {
                    n as Self
                }

                #[cfg(feature = "std")]
                #[inline(always)]
                fn pow_i(self, exp: i32) -> Self {
                    self.powi(exp)
                }

                #[cfg(not(feature = "std"))]
                #[inline(always)]
                fn pow_i(self, exp: i32) -> Self {
                    powi_internal(self, exp)
                }

                #[inline(always)]
                fn cast_int_rep(n: Self::IntRep) -> Self {
                    n as Self
                }
            }
        )*
    }
}

#[allow(dead_code)]
fn powi_internal<F: FloatLike>(mut base: F, n: i32) -> F {
    // TODO: Exponentiation by squaring.
    // Likely possible to optimise since base is always 10.

    if n == 0 {
        return F::ONE;
    }

    let is_neg = n.is_negative();
    let mut n = n.unsigned_abs();

    let mut acc = F::ONE;

    while n > 0 {
        if !n.is_multiple_of(2)  {
            acc = acc.mul(base);
        }

        base = base.mul(base);
        n /= 2;
    }

    if is_neg {
        F::ONE.div(acc)
    } else {
        acc
    }
}

impl_NumLike!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_FloatLike!((f32, u32), (f64, u64));

const SIGN_INFER: u8 = 0;
const SIGN_UNSIGNED: u8 = 1;
const SIGN_SIGNED: u8 = 2;

/// A type containing a parsed integer along with parsing metadata
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct IntWithMeta<O> {
    pub value: O,
    pub digit_count: i32,
    pub is_negative: bool,
    pub overflow_count: i32,
}

/// Helper trait for constructing the return type of [`integer_custom`].
pub trait IntReturn<O, const INCLUDE_METADATA: bool> {
    type Return;
    fn construct(value: O, digit_count: i32, is_negative: bool, overflow_count: i32) -> Self::Return;
}

impl<O> IntReturn<O, true> for () {
    type Return = IntWithMeta<O>;

    #[inline(always)]
    fn construct(value: O, digit_count: i32, is_negative: bool, overflow_count: i32) -> Self::Return {
        IntWithMeta { value, digit_count, is_negative, overflow_count }
    }
}

impl<O> IntReturn<O, false> for () {
    type Return = O;

    #[inline(always)]
    fn construct(value: O, _digit_count: i32, _is_negative: bool, _overflow_count: i32) -> Self::Return {
        value
    }
}

/// Shared helper function for parsing integers.
/// `MANTISSA_MODE` is only useful together with `INCLUDE_METADATA`, since
/// it returns the exponent in the [`IntWithMeta::overflow_count`] field.
const fn integer_internal<const CHECKED: bool,
                          const SIGNED: u8,
                          const LEADING_PLUS: bool,
                          const LEADING_ZEROS: bool,
                          const BASE: u8,
                          const INCLUDE_METADATA: bool,
                          const ALLOW_OVERFLOW: bool,
                          const CONSUME_ONLY: bool,
                          const MANTISSA_MODE: bool,
                          const DECIMAL_COMMA: bool,
                          O: NumLike,
                          A: CharLike,
                          I: SliceLike<AtomRefItem = A>,
                          S,
>() -> impl Parser<I, <() as IntReturn<O, INCLUDE_METADATA>>::Return, S>
where
    (): IntReturn<O, INCLUDE_METADATA>,
{
    create_parser!(s, {
        let mut digit_count = 0;
        let mut overflow_count = 0;
        let mut acc = O::ZERO;

        let (signed, report_neg_only) = if SIGNED == SIGN_INFER {
            (O::SIGNED, MANTISSA_MODE)
        } else if (SIGNED == SIGN_SIGNED) && !O::SIGNED {
            (true, true)
        } else {
            (SIGNED == SIGN_SIGNED, MANTISSA_MODE)
        };

        let max_digits = if CHECKED {
            O::max_safe_digits::<BASE>()
        } else {
            0
        };

        // Using the atom iterator is safe, since all valid characters used for
        // constructing integers are ASCII, and thus at valid boundaries in e.g. `&str`.
        let mut iter = s.input.slice_atom_iter();
        let consume = |acc: &mut O,
                       count: &mut i32,
                       overflow_count: &mut i32,
                       digit: u32,
                       is_negative: bool,
                       check: bool| -> Option<()> {
            if CONSUME_ONLY {
                *count += 1;
                return Some(())
            }

            // Digits are between 0 and 9, so they always fit in all types
            let digit = O::cast_u8(digit as u8);
            let mut tmp = *acc;

            macro_rules! check_overflow {
                ($e:expr) => {
                    if ALLOW_OVERFLOW {
                        let x = if *overflow_count == 0 { $e } else { None };
                        match x {
                            Some(v) => v,
                            None => {
                                *overflow_count = overflow_count.saturating_add(1);
                                return Some(());
                            }
                        }
                    } else {
                        $e?
                    }
                };
            }

            tmp = if check {
                check_overflow!(tmp.checked_mul(O::cast_u8(BASE)))
            } else {
                tmp * O::cast_u8(BASE)
            };

            tmp = if !report_neg_only && is_negative {
                if check {
                    check_overflow!(tmp.checked_sub(digit))
                } else {
                    tmp - digit
                }
            } else if check {
                check_overflow!(tmp.checked_add(digit))
            } else {
                tmp + digit
            };

            *count += 1;
            *acc = tmp;

            Some(())
        };

        let mut first_char = iter.next()?.as_char();

        let (is_negative, leading) = if signed && first_char == '-' {
            first_char = iter.next()?.as_char();
            (true, true)
        } else if LEADING_PLUS && first_char == '+' {
            first_char = iter.next()?.as_char();
            (false, true)
        } else {
            (false, false)
        };

        let first_digit = first_char.to_digit(BASE as u32)?;

        // The first digit fits in all types for all allowed bases.
        consume(&mut acc, &mut digit_count, &mut overflow_count, first_digit, is_negative, false);

        let last_char = loop {
            let Some(c) = iter.next().map(|c| c.as_char()) else {
                break None;
            };

            let Some(digit) = c.to_digit(BASE as u32) else {
                break Some(c);
            };

            let check = CHECKED && (digit_count >= max_digits);
            consume(&mut acc, &mut digit_count, &mut overflow_count, digit, is_negative, check)?;
        };

        if !LEADING_ZEROS && (digit_count > 1) && (first_digit == 0) {
            return None;
        }

        let mut consumed = digit_count + overflow_count + if leading {1} else {0};

        if MANTISSA_MODE {
            // Continue with separator and decimal part
            if last_char == Some(if DECIMAL_COMMA {','} else {'.'}) {

                let mut exp = 0;
                let int_overflow = overflow_count;

                for digit in iter.map_while(|d| d.as_char().to_digit(BASE as u32)) {
                    let check = CHECKED && (digit_count >= max_digits);
                    consume(&mut acc, &mut digit_count, &mut overflow_count, digit, is_negative, check)?;
                    exp += 1;
                }

                if exp == 0 {
                    return None;
                }

                consumed += 1 + exp;

                if !CONSUME_ONLY {
                    if int_overflow != 0 {
                        overflow_count = int_overflow;
                    } else {
                        overflow_count -= exp;
                    }
                }
            }
        }

        (digit_count != Default::default()).then(|| {
            s.input = s.input.slice_from(s.input.slice_idx_from_offset(consumed as usize));
            <() as IntReturn<O, INCLUDE_METADATA>>::construct(acc, digit_count, is_negative, overflow_count)
        })
    })
}

/// Configuration for integer parsing. The instance functions can be used to
/// change the behavior of the parse.
#[derive(Clone, Copy)]
pub struct IntConfig<const CHECKED: bool = true,
                     const SIGNED: u8 = SIGN_INFER,
                     const LEADING_PLUS: bool = false,
                     const LEADING_ZEROS: bool = true,
                     const BASE: u8 = 10,
                     const INCLUDE_METADATA: bool = false,
                     const ALLOW_OVERFLOW: bool = false,
                     const CONSUME_ONLY: bool = false>;

impl IntConfig {
    pub const fn new() -> Self {
        IntConfig
    }
}

impl Default for IntConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CHECKED: bool,
     const SIGNED: u8,
     const LEADING_PLUS: bool,
     const LEADING_ZEROS: bool,
     const BASE: u8,
     const INCLUDE_METADATA: bool,
     const ALLOW_OVERFLOW: bool,
     const CONSUME_ONLY: bool>
     IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY> {

    /// Force parsing of an unsigned integer. Can be used even if the type is signed.
    pub const fn unsigned(self) -> IntConfig<CHECKED, SIGN_UNSIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY> {
        IntConfig
    }

    /// Force parsing of a signed integer. Can be used even if the type is unsigned.
    /// The sign can be obtained by using [`IntConfig::with_metadata`].
    pub const fn signed(self) -> IntConfig<CHECKED, SIGN_SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY> {
        IntConfig
    }

    /// Disallow leading zeros.
    pub const fn no_leading_zero(self) -> IntConfig<CHECKED, SIGNED, LEADING_PLUS, false, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY> {
        IntConfig
    }

    /// Allow a leading plus sign.
    pub const fn leading_plus(self) -> IntConfig<CHECKED, SIGNED, true, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY> {
        IntConfig
    }

    /// Set the numeric base for parsing. Must be between 2 and 36.
    pub const fn base<const NEW_BASE: u8>(self) -> IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, NEW_BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY> {
        assert!(NEW_BASE >= 2 && NEW_BASE <= 36, "Base must be between 2 and 36");
        IntConfig
    }

    /// Include metadata about the parsed integer. The result of the parse will be a [`IntWithMeta`].
    pub const fn with_metadata(self) -> IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, true, ALLOW_OVERFLOW, CONSUME_ONLY> {
        IntConfig
    }

    /// Only consume the integer. This is more performant if the actual return value isn't being
    /// consumed as an integer type (e.g. with [`crate::combinators::get_parsed`])
    pub const fn consume_only(self) -> IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, true> {
        IntConfig
    }
}

impl<const SIGNED: u8,
     const LEADING_PLUS: bool,
     const LEADING_ZEROS: bool,
     const BASE: u8,
     const INCLUDE_METADATA: bool,
     const ALLOW_OVERFLOW: bool>
     IntConfig<true, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW> {

    /// Allow integer overflow. The count for overflow digits can be obtained
    /// by using [`IntConfig::with_metadata`].
    ///
    /// This option is only available if [`IntConfig::unchecked`] hasn't been set.
    pub const fn allow_overflow(self) -> IntConfig<true, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, true> {
        IntConfig
    }
}

impl<const CHECKED: bool,
     const SIGNED: u8,
     const LEADING_PLUS: bool,
     const LEADING_ZEROS: bool,
     const BASE: u8,
     const INCLUDE_METADATA: bool>
     IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, false> {

     /// Disable overflow checking.
     ///
     /// This option is only available if [`IntConfig::allow_overflow`] hasn't been set.
     pub const fn unchecked(self) -> IntConfig<false, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, false> {
         IntConfig
     }
}

/// Parse an integer. The type of the integer will be inferred from the context.
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
                            const SIGNED: u8,
                            const LEADING_PLUS: bool,
                            const LEADING_ZEROS: bool,
                            const BASE: u8,
                            const INCLUDE_METADATA: bool,
                            const ALLOW_OVERFLOW: bool,
                            const CONSUME_ONLY: bool,
                            O: NumLike,
                            A: CharLike,
                            I: SliceLike<AtomRefItem = A>,
                            S
>(_config: IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY>
) -> impl Parser<I, <() as IntReturn<O, INCLUDE_METADATA>>::Return, S>
where
    (): IntReturn<O, INCLUDE_METADATA>
{
    integer_internal::<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY, false, false,_,_,_,_>()
}

/// Parse an integer string.
/// General verison taking a config object. See [`IntConfig`] for more information.
///
/// ### Arguments
/// * `_config` - the configuration object
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::{integer_raw, IntConfig};
///
/// let parse_plus = integer_raw(IntConfig::new().leading_plus());
/// let parse_no_plus = integer_raw(IntConfig::new());
/// let input1 = "+10";
/// let input2 = "10";
///
/// assert_eq!(Some(input1), parse(parse_plus, input1).result);
/// assert_eq!(Some(input2), parse(parse_plus, input2).result);
///
/// assert_eq!(None, parse(parse_no_plus, input1).result);
/// assert_eq!(Some(input2), parse(parse_no_plus, input2).result);
/// ```
#[inline]
pub const fn integer_raw<const CHECKED: bool,
                         const SIGNED: u8,
                         const LEADING_PLUS: bool,
                         const LEADING_ZEROS: bool,
                         const BASE: u8,
                         const INCLUDE_METADATA: bool,
                         const ALLOW_OVERFLOW: bool,
                         const CONSUME_ONLY: bool,
                         A: CharLike,
                         I: SliceLike<AtomRefItem = A>,
                         S
>(_config: IntConfig<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, INCLUDE_METADATA, ALLOW_OVERFLOW, CONSUME_ONLY>
) -> impl Parser<I, I, S>
{
    get_parsed(integer_internal::<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZEROS, BASE, false, true, true, false, false, i32,_,_,_>())
}

/// Parse an integer. The type of the integer will be inferred from the context.
/// This parser will fail if the result does not fit in the inferred integer type.
#[inline]
pub const fn integer<O: NumLike,
                     A: CharLike,
                     I: SliceLike<AtomRefItem = A>,
                     S>() -> impl Parser<I, O, S> {
    integer_custom(IntConfig::new())
}

macro_rules! raw_integer {
    ($f:expr) => {
        create_parser!(s, {
            $f(s.input.as_ref()).map(|(res, rest)| {
                s.input = rest;
                res
            })
        })
    };
}

/// Parse a raw little-endian integer from a byte slice.
pub const fn raw_le_integer<'a,
                            O: NumLike,
                            S>() -> impl Parser<&'a [u8], O, S> {
    raw_integer!(O::from_le_bytes)
}

/// Parse a raw big-endian integer from a byte slice.
pub const fn raw_be_integer<'a,
                            O: NumLike,
                            S>() -> impl Parser<&'a [u8], O, S> {
    raw_integer!(O::from_be_bytes)
}

/// Parse a raw native-endian integer from a byte slice.
pub const fn raw_ne_integer<'a,
                            O: NumLike,
                            S>() -> impl Parser<&'a [u8], O, S> {
    raw_integer!(O::from_ne_bytes)
}

/// Configuration for float parsing. The instance functions can be used to
/// change the behavior of the parse.
#[derive(Clone, Copy)]
pub struct FloatConfig<const CHECKED: bool = true,
                       const SIGNED: u8 = SIGN_SIGNED,
                       const SCI: bool = false,
                       const LEADING_PLUS: bool = false,
                       const LEADING_ZEROS_INT: bool = true,
                       const LEADING_ZEROS_EXP: bool = true,
                       const DECIMAL_COMMA: bool = false,
                       const SATURATE: bool = false,
                       const CONSUME_ONLY: bool = false>;

impl FloatConfig {
    pub const fn new() -> Self {
        FloatConfig
    }
}

impl Default for FloatConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CHECKED: bool,
     const SIGNED: u8,
     const SCI: bool,
     const LEADING_PLUS: bool,
     const LEADING_ZERO_INT: bool,
     const LEADING_ZERO_EXP: bool,
     const DECIMAL_COMMA: bool,
     const SATURATE: bool,
     const CONSUME_ONLY: bool>
     FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {

    /// Do not check for overflow when parsing the float components. This is the fastest option if the input is known
    /// to be within bounds.
    pub const fn unchecked(self) -> FloatConfig::<false, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Do not allow negative numbers.
    pub const fn unsigned(self) -> FloatConfig::<CHECKED, SIGN_UNSIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Enable scientific notation e.g. 1.23e10 or 1.23E10.
    pub const fn scientific(self) -> FloatConfig::<CHECKED, SIGNED, true, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Allow a leading plus sign e.g. +1.23.
    pub const fn leading_plus(self) -> FloatConfig::<CHECKED, SIGNED, SCI, true, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Disallow leading zeros in the integer part e.g. 001.23.
    pub const fn no_leading_zero_int(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, false, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Disallow leading zeros in the exponent part e.g. 1.23e0010.
    pub const fn no_leading_zero_exp(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, false, DECIMAL_COMMA, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Use a decimal comma instead of a decimal point e.g. 1,23.
    pub const fn decimal_comma(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, true, SATURATE, CONSUME_ONLY> {
        FloatConfig
    }

    /// Enable saturation arithmetic. When using this option, a parse can never fail as long as the input
    /// has a valid float structure.
    pub const fn saturate(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, true, CONSUME_ONLY> {
        FloatConfig
    }

    /// Do not calculate a result, but only consume the number. If you don't need the result as a float
    /// this is faster.
    pub const fn consume_only(self) -> FloatConfig::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, true> {
        FloatConfig
    }
}

/// Parse float parts (sign, mantissa and exponent).
/// The type of the float will be inferred from the context.
///
/// See [`FloatConfig`] for more information on how to configure.
///
/// ### Arguments
/// * `_config` - the configuration object
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::{float_parts, FloatParts, FloatConfig};
///
/// let parse_sci = float_parts(FloatConfig::new().scientific());
/// let parse_no_sci = float_parts(FloatConfig::new());
/// let input1 = "1.02e30";
///
/// assert_eq!(Some(FloatParts::<f64> { negative: false, mantissa: 102, exp: 28 }),
///            parse(parse_sci, input1).result);
///
/// assert_eq!(Some(FloatParts::<f64> { negative: false, mantissa: 102, exp: -2 }),
///            parse(parse_no_sci, input1).result);
/// ```
#[inline]
pub const fn float_parts<const CHECKED: bool,
                         const SIGNED: u8,
                         const SCI: bool,
                         const LEADING_PLUS: bool,
                         const LEADING_ZERO_INT: bool,
                         const LEADING_ZERO_EXP: bool,
                         const DECIMAL_COMMA: bool,
                         const SATURATE: bool,
                         const CONSUME_ONLY: bool,
                         O: FloatLike,
                         A: CharLike,
                         A2: CharLike,
                         I: SliceLike<AtomRefItem = A, RefItem = A2>,
                         S>(_config: FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY>)
                       -> impl Parser<I, FloatParts<O>, S> {
    attempt(bind(integer_internal::<CHECKED, SIGNED, LEADING_PLUS, LEADING_ZERO_INT, 10, true, SATURATE, CONSUME_ONLY, true, DECIMAL_COMMA, _,_,_,_>(), |meta| {
        let exp = choose_pure!(SCI;
            true => ({
                let exp = integer_internal::<CHECKED, SIGN_SIGNED, true, LEADING_ZERO_EXP, 10, false, SATURATE, CONSUME_ONLY, false, false,_,_,_,_>()
                    .map(|exp: i32| exp + meta.overflow_count);
                choose!(succeed(item_if(|c: A2| matches!(c.as_char(), 'e' | 'E')));
                    Some(_) => exp,
                    _ => pure!(meta.overflow_count)
                )
            }),
            false => pure!(meta.overflow_count)
        );

        map(exp, move |exp| FloatParts { negative: meta.is_negative, mantissa: meta.value, exp })
    }))
}

/// Parse a float. The type of the float will be inferred from the context.
/// General verison taking a config object. See [`FloatConfig`] for more information.
///
/// This parser is fast but may incur (very) small rounding errors.
/// For exact but slower float parsing, use [`float_exact`].
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
                          const SIGNED: u8,
                          const SCI: bool,
                          const LEADING_PLUS: bool,
                          const LEADING_ZERO_INT: bool,
                          const LEADING_ZERO_EXP: bool,
                          const DECIMAL_COMMA: bool,
                          const SATURATE: bool,
                          const CONSUME_ONLY: bool,
                          O: FloatLike,
                          A: CharLike,
                          A2: CharLike,
                          I: SliceLike<AtomRefItem = A, RefItem = A2>,
                          S>(_config: FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY>)
                          -> impl Parser<I, O, S> {
    map(float_parts::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY,O,_,_,_,_>(_config), |f| {
        f.to_float_fast()
    })
}

/// Parse a floating point number on the form `int.dec`.
/// The type of the number will be inferred from the context.
///
/// This parser will fail if the concatenated integer and decimal part doesn't fit within
/// the integer representation of the float (e.g. `u64` for `f64`).
///
/// This parser is fast but may incur (very) small rounding errors.
/// For exact but slower float parsing, use [`float_exact`].
///
/// For parsing arbirarily long floats, use [`float_custom`] with [`FloatConfig::saturate`]
/// or [`float_exact`].
#[inline]
pub const fn float<O: FloatLike,
                   A: CharLike,
                   A2: CharLike,
                   I: SliceLike<RefItem = A, AtomRefItem = A2>,
                   S>() -> impl Parser<I, O, S> {
    float_custom(FloatConfig::new())
}

/// Parse a float string.
///
/// ### Arguments
/// * `_config` - the configuration object
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::{float_raw, FloatConfig};
///
/// let parse_sci = float_raw(FloatConfig::new().scientific());
/// let input1 = "1.02e30";
///
/// assert_eq!(Some("1.02e30"), parse(parse_sci, input1).result);
/// ```
#[inline]
pub const fn float_raw<const CHECKED: bool,
                       const SIGNED: u8,
                       const SCI: bool,
                       const LEADING_PLUS: bool,
                       const LEADING_ZERO_INT: bool,
                       const LEADING_ZERO_EXP: bool,
                       const DECIMAL_COMMA: bool,
                       const SATURATE: bool,
                       const CONSUME_ONLY: bool,
                       A: CharLike,
                       A2: CharLike,
                       I: SliceLike<AtomRefItem = A, RefItem = A2>,
                       S>(_config: FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY>)
                       -> impl Parser<I, I, S> {
    get_parsed(float_parts::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, true, f32,_,_,_,_>(_config.consume_only()))
}

/// Parse a float using using the stdlib [`FromStr`]. This is slower than [`float`] and
/// [`float_custom`], but does not suffer from rounding errors.
///
/// ### Arguments
/// * `_config` - the configuration object
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::{float_exact, FloatConfig};
///
/// let parse_sci = float_exact(FloatConfig::new().scientific());
/// let input1 = "1.02e30";
///
/// assert_eq!(Some(1.02e30_f64), parse(parse_sci, input1).result);
/// ```
#[inline]
pub const fn float_exact<'a,
                       const CHECKED: bool,
                       const SIGNED: u8,
                       const SCI: bool,
                       const LEADING_PLUS: bool,
                       const LEADING_ZERO_INT: bool,
                       const LEADING_ZERO_EXP: bool,
                       const DECIMAL_COMMA: bool,
                       const SATURATE: bool,
                       const CONSUME_ONLY: bool,
                       A: CharLike,
                       A2: CharLike,
                       I: SliceLike<AtomRefItem = A, RefItem = A2> + Into<&'a str>,
                       O: FloatLike + FromStr,
                       S>(_config: FloatConfig<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY>)
                       -> impl Parser<I, O, S> {
    from_str(float_raw::<CHECKED, SIGNED, SCI, LEADING_PLUS, LEADING_ZERO_INT, LEADING_ZERO_EXP, DECIMAL_COMMA, SATURATE, CONSUME_ONLY,_,_,_,_>(_config))
}

#[cfg(test)]
mod tests {

use core::fmt::{Display, Debug};

use alloc::string::{ToString};

use crate::{combinators::hint_type, core::parse, number::{FloatConfig, IntConfig, IntWithMeta, NumLike, float, float_custom, integer, integer_custom, integer_raw, powi_internal, raw_be_integer, raw_le_integer, raw_ne_integer}};

    #[test]
    fn infer_integer() {
        fn test_int<N: NumLike + Display + Debug>(max_plus_1: &str, min_minus_1: &str) {
            // Zero
            assert_eq!(0, parse(integer(), "0").result.unwrap());

            // Max
            assert_eq!(N::MAX, parse(integer(), N::MAX.to_string().as_str()).result.unwrap());

            // Min
            assert_eq!(N::MIN, parse(integer(), N::MIN.to_string().as_str()).result.unwrap());

            // Max + 1
            assert!((parse(integer(), max_plus_1).result as Option<N>).is_none());

            // Min - 1
            assert!((parse(integer(), min_minus_1).result as Option<N>).is_none());
        }

        test_int::<u8>("256", "-1");
        test_int::<i8>("128", "-129");
        test_int::<u16>("65536", "-1");
        test_int::<i16>("32768", "-32769");
        test_int::<u32>("4294967296", "-1");
        test_int::<i32>("2147483648", "-2147483649");
        test_int::<u64>("18446744073709551616", "-1");
        test_int::<i64>("9223372036854775808", "-9223372036854775809");
        test_int::<u128>("340282366920938463463374607431768211456", "-1");
        test_int::<i128>("170141183460469231731687303715884105728",
            "-170141183460469231731687303715884105729");

        // Test full u16 and i16 range
        for n in u16::MIN..=u16::MAX {
            assert_eq!(n, parse(integer(), n.to_string().as_str()).result.unwrap());
        }

        for n in i16::MIN..=i16::MAX {
            assert_eq!(n, parse(integer(), n.to_string().as_str()).result.unwrap());
        }
    }

    #[test]
    fn zero() {
        let leading_zero = integer();
        assert_eq!(0, parse(leading_zero, "0").result.unwrap());
        assert_eq!(0, parse(leading_zero, "00").result.unwrap());
        assert_eq!(1, parse(leading_zero, "01").result.unwrap());

        let no_leading_zeros = integer_custom(IntConfig::new().no_leading_zero());
        assert_eq!(0, parse(no_leading_zeros, "0").result.unwrap());
        assert!(parse(no_leading_zeros, "00").result.is_none());
        assert!(parse(no_leading_zeros, "01").result.is_none());
    }

    #[test]
    fn integer_override_sign() {
        let p = integer_custom(IntConfig::new().unsigned());
        assert_eq!(127_i8, parse(p, "127").result.unwrap());

        assert!((parse(p, "-0").result as Option<i8>).is_none());
        assert!((parse(p, "128").result as Option<i8>).is_none());
        assert!((parse(p, "-1").result as Option<i8>).is_none());
    }

    #[test]
    fn integer_hex() {
        let p = integer_custom(IntConfig::new().base::<16>());
        assert_eq!(0u8, parse(p, "0").result.unwrap());
        assert_eq!(127, parse(p, "7F").result.unwrap());
        assert_eq!(255, parse(p, "FF").result.unwrap());
        assert!((parse(p, "100").result as Option<u8>).is_none());
        assert!((parse(p, "-1").result as Option<u8>).is_none());

        let p = integer_custom(IntConfig::new().base::<16>());
        assert_eq!(-1i8, parse(p, "-1").result.unwrap());
        assert_eq!(-128, parse(p, "-80").result.unwrap());
        assert_eq!(127, parse(p, "7F").result.unwrap());
        assert!((parse(p, "80").result as Option<i8>).is_none());
        assert!((parse(p, "-81").result as Option<i8>).is_none());
    }

    #[test]
    fn int_overflow_test() {
        let p = integer_custom(IntConfig::new().allow_overflow().with_metadata());
        let edge = "2561";
        let res = parse(p, edge);
        let val: IntWithMeta<u8> = res.result.unwrap();
        assert_eq!(25, val.value);
        assert_eq!(2, val.overflow_count);


        let max = usize::MAX;
        let max_string = max.to_string();

        for i in 1..10 {
            let p = integer_custom(IntConfig::new().allow_overflow().with_metadata());
            let overflow = max_string.clone() + &"0".repeat(i as usize);
            let res = parse(p, overflow.as_str());
            let value = res.result.unwrap();

            assert_eq!(max, value.value);
            assert_eq!(i, value.overflow_count);
            assert_eq!("", res.state);
        }

        let min = isize::MIN;
        let min_string = &min.to_string();

        for i in 1..10 {
            let p = integer_custom(IntConfig::new().allow_overflow().with_metadata());
            let overflow = min_string.clone() + &"0".repeat(i as usize);
            let res = parse(p, overflow.as_str());
            let value = res.result.unwrap();

            assert_eq!(min, value.value);
            assert_eq!(i, value.overflow_count);
            assert_eq!("", res.state);
        }
    }

    #[test]
    fn integer_raw_test() {
        let p = integer_raw(IntConfig::new());
        let input = "1234567";
        let res = parse(p, input);
        let int = res.result.unwrap();
        assert_eq!(int, input);
        assert_eq!(res.state, "");
    }

    #[test]
    fn integer_consume_only_test() {
        let p = hint_type::<IntWithMeta<u64>,_,_>(integer_custom(IntConfig::new().consume_only().with_metadata()));
        let input = "1234567";
        let res = parse(p, input);
        let int = res.result.unwrap();
        assert_eq!(int.value, 0);
        assert_eq!(int.digit_count, input.len() as i32);
        assert_eq!(res.state, "");
    }

    #[test]
    fn float_test() {
        assert_eq!(0_f32, parse(float(), "0").result.unwrap());
        assert_eq!(0.0_f32, parse(float(), "0.0").result.unwrap());
        assert_eq!(100000000_f32, parse(float(), "100000000").result.unwrap());
        assert_eq!(-100000000_f32, parse(float(), "-100000000").result.unwrap());
        assert_eq!(13.37_f32, parse(float(), "13.37").result.unwrap());
        assert_eq!(-13.37_f32, parse(float(), "-13.37").result.unwrap());
        assert_eq!(13.07_f32, parse(float(), "13.07").result.unwrap());
        assert_eq!(-13.07_f32, parse(float(), "-13.07").result.unwrap());
        assert_eq!(1.123_f32, parse(float(), "1.123").result.unwrap());
        assert_eq!(0.001_f32, parse(float(), "0.001").result.unwrap());
        assert_eq!(-0.001_f32, parse(float(), "-0.001").result.unwrap());
    }

    #[test]
    fn float_negative_zero_test() {
        let zero: f32 = parse(float(), "-0").result.unwrap();
        let zero_point_zero: f32 = parse(float(), "-0.0").result.unwrap();
        assert_eq!(-0_f32, zero);
        assert!(zero.is_sign_negative());

        assert_eq!(-0.0_f32, zero_point_zero);
        assert!(zero_point_zero.is_sign_negative());
    }

    #[test]
    fn float_no_partial_test() {
        let p = float_custom(FloatConfig::new().scientific());

        let dec_overflow = "0.".to_string() + usize::MAX.to_string().as_str() + "0";
        let int_overflow = usize::MAX.to_string() + "0.5";

        // Too many digits to fit in usize
        let res = parse(p, int_overflow.as_str());
        assert_eq!(None as Option<f32>, res.result);
        assert_eq!(int_overflow.as_str(), res.state);

        // Too many digits to fit in usize
        let res = parse(p, dec_overflow.as_str());
        assert_eq!(None as Option<f32>, res.result);
        assert_eq!(dec_overflow.as_str(), res.state);

        let wrong_exp = "1234.2134et";
        let res = parse(p, wrong_exp);
        assert_eq!(None as Option<f32>, res.result);
        assert_eq!(wrong_exp, res.state);

        let exp_overflow = "1234.2134e".to_string() + u32::MAX.to_string().as_str() + "0";
        let res = parse(p, exp_overflow.as_str());
        assert_eq!(None as Option<f32>, res.result);
        assert_eq!(exp_overflow.as_str(), res.state);

        // Empty decimal
        let empty_decimal = "1234.";
        let res = parse(p, empty_decimal);
        assert_eq!(None as Option<f32>, res.result);
        assert_eq!(empty_decimal, res.state);

        // Empty exponent
        let empty_exponent = "123.321e";
        let res = parse(p, empty_exponent);
        assert_eq!(None as Option<f32>, res.result);
        assert_eq!(empty_exponent, res.state);
    }

    #[test]
    fn float_saturating_test() {
        // Attempt too long parts, should saturate instead of overflowing
        // Validate against std

        let p = float_custom(FloatConfig::new().scientific().saturate());
        let max_u64_plus_1 = "18446744073709551616";
        assert_eq!(max_u64_plus_1.parse::<f64>().unwrap(), parse(p, max_u64_plus_1).result.unwrap());

        let min_i64_minus_1 = "-9223372036854775809";
        assert_eq!(min_i64_minus_1.parse::<f64>().unwrap(), parse(p, min_i64_minus_1).result.unwrap());

        for additional_digits in 0..30 {
            for sign in ["", "-"] {
                let p = float_custom(FloatConfig::new().scientific().saturate());
                let additional_int = "1".repeat(additional_digits);
                let additional_dec = "1".repeat(additional_digits);
                let int_overflow = sign.to_string() + &usize::MAX.to_string() + &additional_int + ".5";
                let dec_overflow = sign.to_string() + "0." + usize::MAX.to_string().as_str() + &additional_dec;

                let res = parse(p, int_overflow.as_str());
                assert_eq!(int_overflow.parse::<f64>().unwrap(), res.result.unwrap(), "{}", int_overflow);
                assert_eq!("", res.state);

                let res = parse(p, dec_overflow.as_str());
                assert_eq!(dec_overflow.parse::<f64>().unwrap(), res.result.unwrap(), "{}", dec_overflow);
                assert_eq!("", res.state);
            }
        }
    }

    #[test]
    fn float_sci_test() {
        let p = float_custom(FloatConfig::new().scientific());
        assert_eq!(1e10, parse(p, "1e10").result.unwrap());
        assert_eq!(0.05e15, parse(p, "0.05e15").result.unwrap());
        assert_eq!(-1.3e12, parse(p, "-1.3e12").result.unwrap());
        assert_eq!(-1.3e+12, parse(p, "-1.3e+12").result.unwrap());
        assert_eq!(0e400, parse(p, "0e400").result.unwrap());
        assert_eq!("1e400".parse::<f64>().unwrap(), parse(p, "1e400").result.unwrap());

        let zero = parse(p, "0e0").result.unwrap();
        assert_eq!(zero, 0.0);
        assert!(zero.is_sign_positive());

        let minus_zero = parse(p, "-0e0").result.unwrap();
        assert_eq!(minus_zero, 0.0);
        assert!(minus_zero.is_sign_negative());
    }

    #[test]
    fn raw_integer() {
        let empty: &[u8] = &[];
        let tail: &[u8] = &[3, 4];
        for i in i16::MIN..=i16::MAX {
            let le_exact = i.to_le_bytes();
            let be_exact = i.to_be_bytes();
            let ne_exact = i.to_ne_bytes();
            let le = parse(raw_le_integer(), le_exact.as_slice());
            let be = parse(raw_be_integer(), be_exact.as_slice());
            let ne = parse(raw_ne_integer(), ne_exact.as_slice());
            assert_eq!(i, le.result.unwrap());
            assert_eq!(empty, le.state);
            assert_eq!(i, be.result.unwrap());
            assert_eq!(empty, be.state);
            assert_eq!(i, ne.result.unwrap());
            assert_eq!(empty, ne.state);

            let le_larger = &[le_exact[0], le_exact[1], 3, 4];
            let be_larger = &[be_exact[0], be_exact[1], 3, 4];
            let ne_larger = &[ne_exact[0], ne_exact[1], 3, 4];
            let le = parse(raw_le_integer(), le_larger);
            let be = parse(raw_be_integer(), be_larger);
            let ne = parse(raw_ne_integer(), ne_larger);
            assert_eq!(i, le.result.unwrap());
            assert_eq!(tail, le.state);
            assert_eq!(i, be.result.unwrap());
            assert_eq!(tail, be.state);
            assert_eq!(i, ne.result.unwrap());
            assert_eq!(tail, ne.state);
        }

        assert!((parse(raw_be_integer(), &[0x12, 0x34, 0x56, 0x78]).result as Option<u64>).is_none());
    }

    #[test]
    fn powi_internal_test() {
        for i in 0..100 {
            for j in 0..100 {
                assert_eq!((i as f64).powi(j), powi_internal(i as f64, j));
                assert_eq!((i as f64).powi(-j), powi_internal(i as f64, -j));
                assert_eq!((-i as f64).powi(j), powi_internal(-i as f64, j));
                assert_eq!((-i as f64).powi(-j), powi_internal(-i as f64, -j));
            }
        }
    }

    #[test]
    fn max_digits() {
        assert_eq!(8, u8::max_safe_digits::<2>());
        assert_eq!(7, i8::max_safe_digits::<2>());
        assert_eq!(2, u8::max_safe_digits::<10>());
        assert_eq!(2, i8::max_safe_digits::<10>());
        assert_eq!(2, u8::max_safe_digits::<16>());
        assert_eq!(1, i8::max_safe_digits::<16>());

        assert_eq!(16, u16::max_safe_digits::<2>());
        assert_eq!(15, i16::max_safe_digits::<2>());
        assert_eq!(4, u16::max_safe_digits::<10>());
        assert_eq!(4, i16::max_safe_digits::<10>());
        assert_eq!(4, u16::max_safe_digits::<16>());
        assert_eq!(3, i16::max_safe_digits::<16>());

        assert_eq!(32, u32::max_safe_digits::<2>());
        assert_eq!(31, i32::max_safe_digits::<2>());
        assert_eq!(9, u32::max_safe_digits::<10>());
        assert_eq!(9, i32::max_safe_digits::<10>());
        assert_eq!(8, u32::max_safe_digits::<16>());
        assert_eq!(7, i32::max_safe_digits::<16>());

        assert_eq!(64, u64::max_safe_digits::<2>());
        assert_eq!(63, i64::max_safe_digits::<2>());
        assert_eq!(19, u64::max_safe_digits::<10>());
        assert_eq!(18, i64::max_safe_digits::<10>());
        assert_eq!(16, u64::max_safe_digits::<16>());
        assert_eq!(15, i64::max_safe_digits::<16>());

        assert_eq!(128, u128::max_safe_digits::<2>());
        assert_eq!(127, i128::max_safe_digits::<2>());
        assert_eq!(38, u128::max_safe_digits::<10>());
        assert_eq!(38, i128::max_safe_digits::<10>());
        assert_eq!(32, u128::max_safe_digits::<16>());
        assert_eq!(31, i128::max_safe_digits::<16>());
    }
}
