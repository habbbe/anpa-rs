use crate::{slicelike::SliceLike, core::{Parser, AnpaState}};
use core::borrow::Borrow;

#[inline]
pub fn success<I, S>() -> impl Parser<I, (), S> {
    create_parser!(_s, Some(()))
}

#[inline]
pub fn failure<I, S>() -> impl Parser<I, (), S> {
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

#[inline]
pub fn item_if<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I::RefItem, S> {
    create_parser!(s, {
        let first = s.input.slice_first()?;
        let res = if pred(first) {
            s.input = s.input.slice_from(1);
            Some(first)
        } else {
            None
        };
        res
    })
}

#[inline]
pub fn item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I::RefItem, S> {
    item_if(move |c| I::slice_item_eq_ref_item(item.borrow(), c))
}

#[inline]
pub fn not_item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I::RefItem, S> {
    item_if(move |c| !I::slice_item_eq_ref_item(item.borrow(), c))
}

#[inline]
pub fn item_while<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let idx;
        match s.input.slice_find_pred(|x| !pred(x)) {
            None => idx = s.input.slice_len(),
            Some(0) => return None,
            Some(n) => idx = n
        }

        let res;
        (res, s.input) = s.input.slice_split_at(idx);
        Some(res)
    })
}

#[inline]
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

#[inline]
pub fn until_item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    internal_until!(item.borrow(), 1, SliceLike::slice_find)
}

#[inline]
pub fn until_seq<I: SliceLike, B: Borrow<I> + Copy, S>(seq: B) -> impl Parser<I, I, S> {
    internal_until!(*seq.borrow(), seq.borrow().slice_len(), SliceLike::slice_find_seq)
}

#[inline]
pub fn rest<I: SliceLike, S>() -> impl Parser<I, I, S> {
    create_parser!(s, {
        let all;
        (all, s.input) = s.input.slice_split_at(s.input.slice_len());
        Some(all)
    })
}

#[inline]
pub fn empty<I: SliceLike, S>() -> impl Parser<I, I, S> {
    create_parser!(s, {
        s.input.slice_is_empty().then_some(s.input)
    })
}

#[cfg(test)]
mod tests {
    use crate::core::parse;

    use super::item_while;

    #[test]
    fn take_while_test() {
        let p = item_while(|c| c == 'x');
        let res = parse(p, "xxxxy");
        assert_eq!(res.result.unwrap(), "xxxx");
        assert_eq!(res.state, "y");

        let p = item_while(|c: char| c.is_digit(10));
        assert_eq!(parse(p, "1234abcd").result.unwrap(), "1234")
    }
}