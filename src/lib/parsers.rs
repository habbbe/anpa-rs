use crate::{slicelike::SliceLike, core::{Parser, AnpaState}};
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
    ($type:ty, $id:ident) => {
        pub fn $id<'a, S>(radix: u32) -> impl Parser<&'a str, $type, S> {
            create_parser!(s, {
                let mut idx = 0;
                let mut acc: $type = 0;
                for digit in s.input.chars().map_while(|c| c.to_digit(radix)) {
                    acc = acc.checked_mul(radix as $type)?.checked_add(digit as $type)?;
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

internal_integer!(u8, integer_u8);
internal_integer!(u16, integer_u16);
internal_integer!(u32, integer_u32);
internal_integer!(u64, integer_u64);