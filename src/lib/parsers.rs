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
