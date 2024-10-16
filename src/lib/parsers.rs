use crate::{core::Parser, needle::Needle, slicelike::SliceLike};

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

/// Create a parser that parses a single item matching the provided predicate.
///
/// ### Arguments
/// * `pred` - the predicate
#[inline]
pub fn item_if<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I::RefItem, S> {
    create_parser!(s, {
        s.input.slice_first_if(pred).map(|(res, rest)| {
            s.input = rest;
            res
        })
    })
}

/// Create a parser for matching the provided argument input via `==`.
///
/// The element can be anything implementing the trait `Needle` for the parser input.
///
/// ### Arguments
/// * `needle` - the element to match
#[inline]
pub fn elem<O, I: Copy, S>(needle: impl Needle<I, O>) -> impl Parser<I, O, S>{
    elem!(needle)
}

/// Create a parser that parses while the items in the input matches the predicate.
///
/// This parser never fails, so if an empty parse should not be permitted, wrap it in
/// a `not_empty` combinator.
///
/// ### Arguments
/// * `pred` - the predicate
#[inline]
pub fn item_while<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let idx = s.input.slice_find_pred(|x| !pred(x))
            .unwrap_or(s.input.slice_len());

        let res;
        (res, s.input) = s.input.slice_split_at(idx);
        Some(res)
    })
}

/// Create a parser that parses until the input matches the provided argument.
///
/// ### Arguments
/// * `search` - the element to search for
#[inline]
pub fn until<O, I: SliceLike, N: Needle<I, O>, S>(needle: N) -> impl Parser<I, I, S> {
    until!(needle)
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
    use crate::{core::parse, parsers::until};

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

    #[test]
    fn until_test() {
        let p = until('x');
        let res = parse(p, "xxxxy");
        assert_eq!(res.result.unwrap(), "");
        assert_eq!(res.state, "xxxy");

        let p = until("xxx");
        let res = parse(p, "xxxxy");
        assert_eq!(res.result.unwrap(), "");
        assert_eq!(res.state, "xy");

        let p = until("y");
        let res = parse(p, "xxxxy");
        assert_eq!(res.result.unwrap(), "xxxx");
        assert_eq!(res.state, "");
    }
}