use crate::{core::Parser, needle::Needle, prefix::Prefix, slicelike::SliceLike};

/// Create a parser that always succeeds.
#[inline]
pub const fn success<I: SliceLike, S>() -> impl Parser<I, (), S> {
    pure!(())
}

/// Create a parser that always fails.
#[inline]
pub const fn failure<I: SliceLike, O, S>() -> impl Parser<I, O, S> {
    create_parser!(_s, None)
}

/// Create a parser that parses a single item matching the provided predicate.
///
/// ### Consuming
/// On successful parse
///
/// ### Arguments
/// * `pred` - the predicate
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::item_if;
///
/// let parse_uppercase = item_if(|c: char| c.is_uppercase());
/// let input1 = "A";
/// let input2 = "a";
/// assert_eq!(parse(parse_uppercase, input1).result, Some('A'));
/// assert_eq!(parse(parse_uppercase, input2).result, None);
/// ```
#[inline]
pub const fn item_if<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I::RefItem, S> {
    create_parser!(s, {
        s.input.slice_first_if(pred).map(|(res, rest)| {
            s.input = rest;
            res
        })
    })
}

/// Create a parser that parses a single item.
///
/// ### Consuming
/// Always
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::item;
///
/// let parse_item = item();
/// let input1 = "x";
/// let input2 = "";
/// assert_eq!(parse(parse_item, input1).result, Some('x'));
/// assert_eq!(parse(parse_item, input2).result, None);
/// ```
#[inline]
pub const fn item<I: SliceLike, S>() -> impl Parser<I, I::RefItem, S> {
    item_if(|_| true)
}

/// Create a parser for matching the provided prefix.
/// Returns the parsed prefix on success.
///
/// If the result is not needed, use [`skip()`] instead.
///
/// The prefix can be anything implementing the [`Prefix`] trait for the parser input.
/// Implementations are provided for single elements and sequences of both `&str`
/// and `&[T]`.
///
/// For performance tuning, consider using the inlined version [`take!`].
///
/// ### Consuming
/// Consumes prefix on successful parse
///
/// ### Arguments
/// * `prefix` - the prefix to match
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::take;
///
/// let parse_single = take('a');
/// let parse_seq = take("abc");
/// let input = "abcd";
/// assert_eq!(parse(parse_single, input).result, Some('a'));
/// assert_eq!(parse(parse_seq, input).result, Some("abc"));
/// ```
#[inline]
pub const fn take<I: SliceLike, O, S>(prefix: impl Prefix<I, O>) -> impl Parser<I, O, S>{
    take!(prefix)
}

/// Create a parser for matching the provided prefix.
///
/// For better performance, this parser should be used if the result isn't saved
/// or inspected.
///
/// The prefix can be anything implementing the [`Prefix`] trait for the parser input.
/// Implementations are provided for single elements and sequences of both `&str`
/// and `&[T]`.
///
/// For performance tuning, consider using the inlined version [`skip!`].
///
/// ### Consuming
/// Consumes prefix on successful parse
///
/// ### Arguments
/// * `prefix` - the prefix to match
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::skip;
///
/// let parse_single = skip('a');
/// let parse_seq = skip("abc");
/// let input = "abcd";
/// assert_eq!(parse(parse_single, input).result, Some(()));
/// assert_eq!(parse(parse_seq, input).result, Some(()));
/// ```
#[inline]
pub const fn skip<I: SliceLike, O, S>(prefix: impl Prefix<I, O>) -> impl Parser<I, (), S>{
    skip!(prefix)
}

/// Create a parser that parses while the items in the input matches the predicate.
///
/// This parser never fails, so if an empty parse should not be permitted, wrap it in
/// a [`not_empty`](crate::combinators::not_empty) combinator.
///
/// ### Consuming
/// Consumes all matched items.
///
/// ### Arguments
/// * `pred` - the predicate
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::item_while;
///
/// let parse_odd = item_while(|n: &u8| n % 2 != 0);
/// let input: &[u8] = &[7, 5, 3, 2];
/// assert_eq!(parse(parse_odd, input).result, Some([7, 5, 3].as_slice()));
/// ```
#[inline]
pub const fn item_while<I: SliceLike, S>(pred: impl FnOnce(I::RefItem) -> bool + Copy) -> impl Parser<I, I, S> {
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
/// On a successful parse, all items until the matching needle will be returned.
///
/// The argument can be anything implementing the [`Needle`] trait for the parser input.
/// Implementations are provided for single elements and sequences of both `&str`
/// and `&[T]`.
///
/// For performance tuning, consider using the inlined version [`until!`].
///
/// ### Consuming
/// Consumes all items before the matching needle, and the needle itself.
///
/// ### Arguments
/// * `needle` - the needle to search for
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::until;
///
/// let parse_statement = until(';');
/// let input = "let x = 2;";
/// assert_eq!(parse(parse_statement, input).result, Some("let x = 2"));
/// ```
#[inline]
pub const fn until<O, I: SliceLike, N: Needle<I, O>, S>(needle: N) -> impl Parser<I, I, S> {
    until!(needle)
}

/// Create a parser that parses the rest of the input. This parser can never fail.
///
/// ### Consuming
/// All input.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::rest;
///
/// let parse_rest = rest();
/// let input = "everything that is left";
/// assert_eq!(parse(parse_rest, input).result, Some(input));
/// ```
#[inline]
pub const fn rest<I: SliceLike, S>() -> impl Parser<I, I, S> {
    create_parser!(s, {
        let all;
        (all, s.input) = s.input.slice_split_at(s.input.slice_len());
        Some(all)
    })
}

/// Create a parser that is successful only if the input is empty.
/// Returns the empty input on success.
///
/// ### Consuming
/// Nothing
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::empty;
///
/// let parse_empty = empty();
/// let input1 = "";
/// let input2 = ".";
/// assert_eq!(parse(parse_empty, input1).result, Some(""));
/// assert_eq!(parse(parse_empty, input2).result, None);
/// ```
#[inline]
pub const fn empty<I: SliceLike, S>() -> impl Parser<I, I, S> {
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