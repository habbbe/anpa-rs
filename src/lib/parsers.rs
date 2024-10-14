use crate::{slicelike::SliceLike, core::Parser};
use core::borrow::Borrow;

/// Create a parser that always succeeds.
#[inline]
pub fn success<I, S>() -> impl Parser<I, (), S> {
    pure!(())
}

/// Create a parser that always fails.
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

/// Create a parser that parses a single item matching the provided predicate.
///
/// ### Arguments
/// * `pred` - the predicate
#[inline]
pub fn item_if<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I::RefItem, S> {
    create_parser!(s, {
        let first = s.input.slice_first()?;
        pred(first).then(|| {
            s.input = s.input.slice_from(I::slice_size_of_ref_item(first));
            first
        })
    })
}

/// Create a parser for a single item matching the input via `==`.
///
/// ### Arguments
/// * `item` - the item to match
#[inline]
pub fn item<I: SliceLike, B: Into<I::Item> + Copy, S>(item: B) -> impl Parser<I, I::RefItem, S> {
    item_if(move |c| I::slice_item_eq_ref_item(&item.into(), c))
}

/// Create a parser for a single item not matching the input via `==`.
///
/// ### Arguments
/// * `item` - the item to _not_ match
#[inline]
pub fn not_item<I: SliceLike, B: Into<I::Item> + Copy, S>(item: B) -> impl Parser<I, I::RefItem, S> {
    item_if(move |c| !I::slice_item_eq_ref_item(&item.into(), c))
}

/// Create a parser that parses while the items in the input matches the predicate.
///
/// ### Arguments
/// * `pred` - the predicate
#[inline]
pub fn item_while<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let idx = match s.input.slice_find_pred(|x| !pred(x)) {
            None => s.input.slice_len(),
            Some(0) => return None,
            Some(n) => n
        };

        let res;
        (res, s.input) = s.input.slice_split_at(idx);
        Some(res)
    })
}

/// Create a parser for a sequence of items.
///
/// ### Arguments
/// * `items` - the items to match
#[inline]
pub fn seq<I: SliceLike, B: Borrow<I> + Copy, S>(seq: B) -> impl Parser<I, I, S> {
    internal_starts_with!(*seq.borrow(), seq.borrow().slice_len(), SliceLike::slice_starts_with_seq)
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

/// Create a parser that parses until one item in the input matches the predicate.
///
/// ### Arguments
/// * `item` - the item to match
#[inline]
pub fn until_item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    internal_until!(item.borrow(), I::slice_size_of_item(item.borrow()), SliceLike::slice_find)
}

/// Create a parser that parses until a sequence in the input matches the predicate.
///
/// ### Arguments
/// * `seq` - the sequence to match
#[inline]
pub fn until_seq<I: SliceLike, B: Borrow<I> + Copy, S>(seq: B) -> impl Parser<I, I, S> {
    internal_until!(*seq.borrow(), seq.borrow().slice_len(), SliceLike::slice_find_seq)
}

/// Create a parser that parses the rest of the input. This parser can never fail.
#[inline]
pub fn rest<I: SliceLike, S>() -> impl Parser<I, I, S> {
    create_parser!(s, {
        let all;
        (all, s.input) = s.input.slice_split_at(s.input.slice_len());
        Some(all)
    })
}

/// Create a parser that is successful only if the input is empty.
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