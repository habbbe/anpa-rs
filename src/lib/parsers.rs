use crate::{slicelike::SliceLike, core::{Parser, AnpaState}, combinators::{succeed, bind}};
use core::borrow::Borrow;

pub fn success<I: SliceLike, S>() -> impl Parser<I, (), S> {
    create_parser!(_s, Some(()))
}

pub fn failure<I: SliceLike, S>() -> impl Parser<I, (), S> {
    create_parser!(_s, None)
}

macro_rules! internal_starts_with {
    ($start:expr, $len:expr, $f:expr) => {
        create_parser!(s, {
            if $f(s.input, $start) {
                let res;
                (res, s.input) = s.input.slice_split_at($len);
                Some(res)
            } else {
                None
            }
        })
    }
}

pub fn item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    internal_starts_with!(item, 1, SliceLike::slice_starts_with)
}

pub fn seq<I: SliceLike, B: Borrow<I> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    internal_starts_with!(*item.borrow(), item.borrow().slice_len(), SliceLike::slice_starts_with_seq)
}

macro_rules! internal_until {
    ($item:expr, $len:expr, $f:expr) => {
        create_parser!(s, {
            let index = $f(s.input, $item)?;
            let res = s.input.slice_to(index);
            s.input = s.input.slice_from(index + $len);
            Some(res)
        })
    }
}

pub fn until_item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    internal_until!(item.borrow(), 1, SliceLike::slice_find)
}

pub fn until_seq<I: SliceLike, B: Borrow<I> + Copy, S>(seq: B) -> impl Parser<I, I, S> {
    internal_until!(*seq.borrow(), seq.borrow().slice_len(), SliceLike::slice_find_seq)
}

pub fn rest<I: SliceLike, S>() -> impl Parser<I, I, S> {
    create_parser!(s, {
        let all;
        (all, s.input) = s.input.slice_split_at(s.input.slice_len());
        Some(all)
    })
}

pub fn empty<I: SliceLike, S>() -> impl Parser<I, (), S> {
    create_parser!(s, {
        s.input.slice_is_empty().then_some(())
    })
}

macro_rules! internal_integer {
    ($type:ty, $id:ident, $checked:expr, $neg:expr) => {
        pub fn $id<'a, S>(radix: u32) -> impl Parser<&'a str, $type, S> {
            create_parser!(s, {
                let mut idx = 0;
                let mut acc: $type = 0;
                for digit in s.input.chars().map_while(|c| c.to_digit(radix)) {

                    let digit = digit as $type;
                    let radix = radix as $type;

                    if $checked {
                        acc = acc.checked_mul(radix)?;
                    } else {
                        acc = acc * radix;
                    }

                    if $neg {
                        if $checked {
                            acc = acc.checked_sub(digit)?;
                        } else {
                            acc = acc - digit;
                        }
                    } else {
                        if $checked {
                            acc = acc.checked_add(digit)?;
                        } else {
                            acc = acc + digit;
                        }
                    }
                    idx += 1;
                }

                if idx == 0 {
                    None
                } else {
                    s.input = s.input.slice_from(idx);
                    Some(acc)
                }
            })
        }
    }
}

internal_integer!(u8, integer_u8_checked, true, false);
internal_integer!(u16, integer_u16_checked, true, false);
internal_integer!(u32, integer_u32_checked, true, false);
internal_integer!(u64, integer_u64_checked, true, false);

internal_integer!(u8, integer_u8, false, false);
internal_integer!(u16, integer_u16, false, false);
internal_integer!(u32, integer_u32, false, false);
internal_integer!(u64, integer_u64, false, false);


macro_rules! internal_signed_integer {
    ($type:ty, $id:ident, $checked:expr) => {
        pub fn $id<'a, S>(radix: u32) -> impl Parser<&'a str, $type, S> {
            bind(succeed(item('-')), move |x| {
                create_parser!(s, {
                    if x.is_some() {
                        internal_integer!($type, helper_fun_neg, $checked, true);
                        helper_fun_neg(radix)(s)
                    } else {
                        internal_integer!($type, helper_fun_pos, $checked, false);
                        helper_fun_pos(radix)(s)
                    }
                })
            })
        }
    }
}

internal_signed_integer!(i8, integer_i8, false);
internal_signed_integer!(i16, integer_i16, false);
internal_signed_integer!(i32, integer_i32, false);
internal_signed_integer!(i64, integer_i64, false);

internal_signed_integer!(i8, integer_i8_checked, true);
internal_signed_integer!(i16, integer_i16_checked, true);
internal_signed_integer!(i32, integer_i32_checked, true);
internal_signed_integer!(i64, integer_i64_checked, true);

#[cfg(test)]
mod tests {
    use crate::{core::parse, parsers::{integer_i8, integer_i8_checked, integer_u8, integer_u8_checked}};

    #[test]
    fn unsigned_integer() {
        assert_eq!(parse(integer_u8(10), "0").1.unwrap(), 0);
        assert_eq!(parse(integer_u8(10), "127").1.unwrap(), 127);
        assert_eq!(parse(integer_u8(10), "255").1.unwrap(), 255);
        assert!(parse(integer_u8(10), "-1").1.is_none());

        assert!(parse(integer_u8_checked(10), "256").1.is_none());

        assert_eq!(parse(integer_u8(16), "0").1.unwrap(), 0);
        assert_eq!(parse(integer_u8(16), "F").1.unwrap(), 15);
        assert_eq!(parse(integer_u8(16), "10").1.unwrap(), 16);
        assert_eq!(parse(integer_u8(16), "FF").1.unwrap(), 255);
    }

    #[test]
    fn signed_integer() {
        assert_eq!(parse(integer_i8(10), "0").1.unwrap(), 0);
        assert_eq!(parse(integer_i8(10), "127").1.unwrap(), 127);
        assert_eq!(parse(integer_i8(10), "-1").1.unwrap(), -1);
        assert_eq!(parse(integer_i8(10), "-128").1.unwrap(), -128);

        assert!(parse(integer_i8_checked(10), "-129").1.is_none());
        assert!(parse(integer_i8_checked(10), "128").1.is_none());
    }
}