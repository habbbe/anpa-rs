#[cfg(feature = "std")]
use std::{collections::{BTreeMap, HashMap}, vec::Vec, hash::Hash};

use crate::{core::{AnpaState, Parser}, parsers::success, slicelike::SliceLike};

/// Create a new parser by taking the result of `p`, and applying `f`.
/// This can be used to create a new parser based on the result of another.
///
/// Also available as an extension function: [`bind`](crate::core::ParserExt::bind)
///
/// ### See also
/// Macro [`choose!`] if you want to create different parser for different
/// results.
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the function to generate the new parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::item;
/// use anpa::number::integer;
/// use anpa::combinators::{bind, times};
///
/// let parse_xes = bind(integer(), |n: u32| times(n, item()));
/// let input1 = "1xxx";
/// let input2 = "3xxx";
/// let input3 = "xxx";
/// assert_eq!(parse(parse_xes, input1).result, Some("x"));
/// assert_eq!(parse(parse_xes, input2).result, Some("xxx"));
/// assert_eq!(parse(parse_xes, input3).result, None);
/// ```
#[inline]
pub fn bind<I:SliceLike, O1, O2, P, S>(p: impl Parser<I, O1, S>,
                                       f: impl FnOnce(O1) -> P + Copy
) -> impl Parser<I, O2, S> where P: Parser<I, O2, S> {
    create_parser!(s, f(p(s)?)(s))
}

/// Create a new parser by applying a transformation `f` to the result of `p`.
/// This differs from [`bind`] in that the transformation does not return a new
/// parser. Use this combinator if you just want to modify the result.
///
/// Also available as variadic macro.
///
/// Also available as an extension function: [`map`](crate::core::ParserExt::map)
///
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the transformation function.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::number::integer;
/// use anpa::combinators::map;
///
/// let parse_next_even = map(integer(), |n: u32| if n % 2 == 0 { n } else {n + 1});
/// let input1 = "1";
/// let input2 = "2";
/// let input3 = "3";
/// assert_eq!(parse(parse_next_even, input1).result, Some(2));
/// assert_eq!(parse(parse_next_even, input2).result, Some(2));
/// assert_eq!(parse(parse_next_even, input3).result, Some(4));
/// ```
#[inline]
pub fn map<I: SliceLike, O, O2, S>(p: impl Parser<I, O, S>,
                                   f: impl FnOnce(O) -> O2 + Copy
) -> impl Parser<I, O2, S> {
    map!(f, p)
}

/// Create a new parser by applying a transformation `f` to the result of `p`.
/// Unlike `map`, this combinator allows optional rejection of the parse by returning
/// `Some` or `None` in the transformation.
///
/// Also available as variadic macro.
///
/// Also available as an extension function: [`map_if`](crate::core::ParserExt::map_if)
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the transformation function.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::item;
/// use anpa::combinators::map_if;
///
/// let parse_binary = map_if(item(), |c: char| {
///     match c {
///         '0' => Some("zero"),
///         '1' => Some("one"),
///         _   => None
///     }
/// });
/// let input1 = "0";
/// let input2 = "1";
/// let input3 = "2";
/// assert_eq!(parse(parse_binary, input1).result, Some("zero"));
/// assert_eq!(parse(parse_binary, input2).result, Some("one"));
/// assert_eq!(parse(parse_binary, input3).result, None);
/// ```
#[inline]
pub fn map_if<I: SliceLike, O, O2, S>(p: impl Parser<I, O, S>,
                                      f: impl FnOnce(O) -> Option<O2> + Copy
) -> impl Parser<I, O2, S> {
    create_parser!(s, { p(s).and_then(f) })
}

/// Transform the parser `p` into a parser with a different result by means of [`Into`].
/// The existing type must implement [`Into<T>`] for the requested type `T`.
///
/// Also available as an extension function: [`into_type`](crate::core::ParserInto::into_type)
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::parsers::item_while;
/// use anpa::combinators::into_type;
///
/// let parse_digits = into_type(item_while(|c: char| c.is_ascii_digit()));
///
/// let input = "1234";
///
/// // Type of result is inferred from the type annotation. Since `Into<String>`
/// // is implemented for `&str`, this is valid.
/// let result1: Option<String> = parse(parse_digits, input).result;
/// assert_eq!(result1, Some("1234".to_owned()));
/// ```
#[inline]
pub fn into_type<I: SliceLike, O: Into<T>, T, S>(p: impl Parser<I, O, S>) -> impl Parser<I, T, S> {
    map(p, O::into)
}

/// Accept or reject the parse based on the predicate `f`.
///
/// Also available as an extension function: [`filter`](crate::core::ParserExt::filter)
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the predicate
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::filter;
/// use anpa::number::integer;
///
/// let parse_even = filter(integer(), |n: &u32| *n % 2 == 0);
///
/// let input1 = "1";
/// let input2 = "2";
///
/// assert_eq!(parse(parse_even, input1).result, None);
/// assert_eq!(parse(parse_even, input2).result, Some(2));
/// ```
#[inline]
pub fn filter<I: SliceLike, O, S>(p: impl Parser<I, O, S>,
                                  f: impl FnOnce(&O) -> bool + Copy
) -> impl Parser<I, O, S> {
    create_parser!(s, p(s).filter(f))
}

/// Transform a parser to a parser that always succeeds. The resulting parser will
/// have its result type changed to `Option`, to allow for introspection of the result
/// of the parse.
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::succeed;
/// use anpa::number::integer;
///
/// let parse_optional_int = succeed(integer());
///
/// let input1 = "123";
/// let input2 = "abc";
///
/// assert_eq!(parse(parse_optional_int, input1).result, Some(Some(123)));
/// assert_eq!(parse(parse_optional_int, input2).result, Some(None));
/// ```
#[inline]
pub fn succeed<I:SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, Option<O>, S> {
    create_parser!(s, {
        Some(p(s))
    })
}

/// Transform a parser to a parser that does not consume any input.
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::peek;
/// use anpa::number::integer;
///
/// let parse_int = peek(integer());
///
/// let input = "123";
///
/// let result = parse(parse_int, input);
///
/// assert_eq!(result.result, Some(123));
/// assert_eq!(result.state, input);
/// ```
#[inline]
pub fn peek<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    create_parser!(s, {
        let pos = s.input;
        let res = p(s);
        s.input = pos;
        res
    })
}

/// Transform a parser to a parser that only succeeds if the parsed sequence is not empty.
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::not_empty;
/// use anpa::parsers::item_while;
///
/// let parse_digits = not_empty(item_while(|c: char| c.is_ascii_digit()));
///
/// let input1 = "123";
/// let input2 = "";
///
/// assert_eq!(parse(parse_digits, input1).result, Some("123"));
/// assert_eq!(parse(parse_digits, input2).result, None);
/// ```
#[inline]
pub fn not_empty<I: SliceLike, O: SliceLike, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    filter(p, |r| !r.slice_is_empty())
}

/// Transform a parser to a parser that does not consume any input on failure.
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::{attempt, filter};
/// use anpa::number::integer;
/// use anpa::parsers::item_while;
///
/// let parse_even = attempt(filter(integer(), |c| c % 2 == 0));
///
/// let input1 = "1234";
/// let input2 = "4321";
///
/// let result1 = parse(parse_even, input1);
/// let result2 = parse(parse_even, input2);
///
/// assert_eq!(result1.result, Some(1234));
/// assert_eq!(result1.state, "");
/// assert_eq!(result2.result, None);
/// assert_eq!(result2.state, "4321");
/// ```
#[inline]
pub fn attempt<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    create_parser!(s, {
        let pos = s.input;
        let res = p(s);
        if res.is_none() {
            s.input = pos;
        }
        res
    })
}

/// Transform a parser to a parser that along with its result also returns how many items that
/// were parsed.
///
/// Note: For `&str`, the number of bytes parsed will be returned
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::count_consumed;
/// use anpa::number::integer;
///
/// let parse_int = count_consumed(integer());
///
/// let input = "1234";
///
/// assert_eq!(parse(parse_int, input).result, Some((4, 1234)));
/// ```
#[inline]
pub fn count_consumed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (I::Idx, O), S> {
    create_parser!(s, {
        let old = s.input.slice_len();
        let res = p(s)?;
        let count = old - s.input.slice_len();
        Some((count, res))
    })
}

/// Transform a parser to a parser that along with its result also returns the input that
/// was parsed.
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::and_parsed;
/// use anpa::number::integer;
///
/// let parse_int = and_parsed(integer());
///
/// let input = "1234";
///
/// assert_eq!(parse(parse_int, input).result, Some((input, 1234)));
/// ```
#[inline]
pub fn and_parsed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (I, O), S> {
    create_parser!(s, {
        let old_input = s.input;
        let res = p(s)?;
        Some((old_input.slice_to(old_input.slice_len() - s.input.slice_len()), res))
    })
}

/// Transform a parser to a parser that ignores its result and instead returns the input that
/// was parsed to produce the result.
///
/// ### Arguments
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::get_parsed;
/// use anpa::combinators::right;
/// use anpa::parsers::skip;
///
/// let parse_abc_then_123 = get_parsed(right(skip("abc"), skip("123")));
///
/// let input = "abc123";
///
/// assert_eq!(parse(parse_abc_then_123, input).result, Some(input));
/// ```
#[inline]
pub fn get_parsed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let old_input = s.input;
        p(s)?;
        Some(old_input.slice_to(old_input.slice_len() - s.input.slice_len()))
    })
}

/// Transform a parser to a parser that only succeeds if it can be applied `times` times without
/// failure.
///
/// ### Arguments
/// * `times` - the number of times to apply `p`
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::times;
/// use anpa::parsers::item_if;
///
/// let parse_4_digits = times(4, (item_if(|c: char| c.is_ascii_digit())));
///
/// let input1 = "1234";
/// let input2 = "123";
///
/// assert_eq!(parse(parse_4_digits, input1).result, Some("1234"));
/// assert_eq!(parse(parse_4_digits, input2).result, None);
/// ```
#[inline]
pub fn times<I: SliceLike, O, S>(times: u32, p: impl Parser<I, O, S>) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let old_input = s.input;
        for _ in 0..times {
            p(s)?;
        }
        Some(old_input.slice_to(old_input.slice_len() - s.input.slice_len()))
    })
}

/// Combine one parser with another, while ignoring the result of the former.
/// The second parser will only be attempted if the first succeeds.
///
/// Also available as variadic macro.
///
/// ### Arguments
/// * `p1` - the first parser (result will be ignored)
/// * `p2` - the second parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::right;
/// use anpa::parsers::{skip, take};
///
/// let parse_abc_then_123 = right(skip("abc"), take("123"));
///
/// let input = "abc123";
///
/// assert_eq!(parse(parse_abc_then_123, input).result, Some("123"));
/// ```
#[inline]
pub fn right<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                      p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O2, S> {
    create_parser!(s, {
        p1(s).and_then(|_| p2(s))
    })
}

/// Combine one parser with another, while ignoring the result of the latter.
/// The second parser will only be attempted if the first succeeds.
///
/// Also available as variadic macro.
///
/// ### Arguments
/// * `p1` - the first parser
/// * `p2` - the second parser (result will be ignored)
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::left;
/// use anpa::parsers::{skip, take};
///
/// let parse_abc_then_123 = left(take("abc"), skip("123"));
///
/// let input = "abc123";
///
/// assert_eq!(parse(parse_abc_then_123, input).result, Some("abc"));
/// ```
#[inline]
pub fn left<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                     p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O1, S> {
    create_parser!(s, {
        p1(s).and_then(|res| p2(s).map(|_| res))
    })
}

/// Combine three parsers, returning the result of the middle one.
///
/// ### Arguments
/// * `p1` - the first parser (result will be ignored)
/// * `p2` - the second parser
/// * `p3` - the third parser (result will be ignored)
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::middle;
/// use anpa::parsers::{skip, take};
///
/// let parse_middle = middle(skip("abc"), take("123"), skip("def"));
///
/// let input = "abc123def";
///
/// assert_eq!(parse(parse_middle, input).result, Some("123"));
/// ```
#[inline]
pub fn middle<I: SliceLike, S, O1, O2, O3>(p1: impl Parser<I, O1, S>,
                                           p2: impl Parser<I, O2, S>,
                                           p3: impl Parser<I, O3, S>
) ->  impl Parser<I, O2, S> {
    right(p1, left(p2, p3))
}

macro_rules! internal_or {
    ($id:ident, $allow_partial:tt, $comment:tt) => {
        /// Create a parser that first tries the one parser `p1`, and if it fails, tries the second parser
        /// `p2`.
        /// Both parsers must have the same result type.
        ///
        /// Also available as variadic macro.
        ///
        #[doc=$comment]
        ///
        /// ### Arguments
        /// * `p1` - the first parser
        /// * `p2` - the second parser
        ///
        /// ### Example
        /// ```
        /// use anpa::core::*;
        /// use anpa::combinators::get_parsed;
        /// use anpa::combinators::or;
        /// use anpa::parsers::{skip, take};
        ///
        /// let parse_abc_or_123 = or(take("abc"), take("123"));
        ///
        /// let input1 = "abc123";
        /// let input2 = "123abc";
        /// let input3 = "a1b2c3";
        ///
        /// assert_eq!(parse(parse_abc_or_123, input1).result, Some("abc"));
        /// assert_eq!(parse(parse_abc_or_123, input2).result, Some("123"));
        /// assert_eq!(parse(parse_abc_or_123, input3).result, None);
        /// ```
        #[inline]
        pub fn $id<I: SliceLike, O, S>(p1: impl Parser<I, O, S>,
                                       p2: impl Parser<I, O, S>
        ) -> impl Parser<I, O, S> {
            create_parser!(s, {
                let pos = s.input;
                p1(s).or_else(|| {
                    if !$allow_partial && s.input.slice_len() != pos.slice_len() {
                        None
                    } else {
                        s.input = pos;
                        p2(s)
                    }
                })
            })
        }
    }
}

internal_or!(or, true, "");
internal_or!(or_no_partial, false, "This differs from `or` in that it will not attempt to use `p2` in case there was any consumed input while processing `p1`.");

macro_rules! internal_or_diff {
    ($id:ident, $allow_partial:tt, $comment:tt) => {
        /// Create a parser that first tries the one parser `p1`, and if it fails, tries the second parser
        /// `p2`.
        /// This version of `or` can accept parsers with different result types, and will therefore have
        /// a result type of `()`.
        ///
        /// Also available as variadic macro.
        ///
        #[doc=$comment]
        ///
        /// ### Arguments
        /// * `p1` - the first parser
        /// * `p2` - the second parser
        ///
        /// ### Example
        /// ```
        /// use anpa::core::*;
        /// use anpa::combinators::or_diff;
        /// use anpa::number::integer;
        /// use anpa::parsers::take;
        ///
        /// let parse_int_or_abc = or_diff(integer().map(|n: u32| n), take("abc"));
        ///
        /// let input1 = "abc123";
        /// let input2 = "123abc";
        /// let input3 = "a1b2c3";
        ///
        /// assert_eq!(parse(parse_int_or_abc, input1).result, Some(()));
        /// assert_eq!(parse(parse_int_or_abc, input2).result, Some(()));
        /// assert_eq!(parse(parse_int_or_abc, input3).result, None);
        /// ```
        #[inline]
        pub fn $id<I: SliceLike, O1, O2, S>(p1: impl Parser<I, O1, S>,
                                            p2: impl Parser<I, O2, S>
        ) -> impl Parser<I, (), S> {
            create_parser!(s, {
                let pos = s.input;
                if p1(s).is_some() {
                    Some(())
                } else {
                    if (!$allow_partial && s.input.slice_len() != pos.slice_len()) {
                        None
                    } else {
                        s.input = pos;
                        p2(s).map(|_| ())
                    }
                }
            })
        }
    }
}

internal_or_diff!(or_diff, true, "");
internal_or_diff!(or_diff_no_partial, false, "This differs from `or_diff` in that it will not attempt to use `p2` in case there was any consumed input while processing `p1`.");

/// Create a parser that allows for using and modifying the user state while transforming
/// the result.
///
/// ### Arguments
/// * `f` - a transformation function that is also allowed to use and modify the user state.
/// * `p` - the parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::lift_to_state;
/// use anpa::number::integer;
///
/// let mut nums = vec![];
/// let parse_middle = lift_to_state(|v: &mut Vec<_>, num: u32| v.push(num), integer());
///
/// let input = "123";
///
/// // Result was transformed to `()` due to return type of `push`.
/// assert_eq!(parse_state(parse_middle, input, &mut nums).result, Some(()));
/// assert_eq!(nums, vec![123]);
/// ```
#[inline]
pub fn lift_to_state<I: SliceLike, S, O1, O2>(f: impl FnOnce(&mut S, O1) -> O2 + Copy,
                                              p: impl Parser<I, O1, S>
) -> impl Parser<I, O2, S> {
    create_parser!(s, {
        p(s).map(|res| f(s.user_state, res))
    })
}

/// Only for use with the `many` family of combinators. Use this function to create the separator
/// argument when parsing multiple elements.
///
/// ### Arguments
/// * `p` - a parser for the separator
/// * `allow_trailing` - whether a trailing separator is allowed.
#[inline]
pub fn separator<I, O, S>(p: impl Parser<I, O, S>, allow_trailing: bool) -> Option<(bool, impl Parser<I, O, S>)> {
    Some((allow_trailing, p))
}

/// Only for use with the `many` family of combinators. Use this function to create the separator
/// argument when no separator should be present.
#[allow(unreachable_code)]
#[inline]
pub fn no_separator<I: SliceLike, S>() -> Option<(bool, impl Parser<I, (), S>)> {
    return None;

    // Unreachable, but provides type/size information about the return value
    Some((false, success()))
}

#[inline(always)]
fn many_internal<I: SliceLike, O, O2, S>(
    s: &mut AnpaState<I, S>,
    p: impl Parser<I, O, S>,
    mut f: impl FnMut(O),
    allow_empty: bool,
    separator: Option<(bool, impl Parser<I, O2, S>)>
) -> bool {
    let mut successes = false;
    let mut has_trailing_sep = false;

    while let Some(res) = p(s) {
        has_trailing_sep = false;
        successes = true;
        f(res);

        if let Some((_, sep)) = separator {
            if sep(s).is_none() {
                break;
            }
            has_trailing_sep = true;
        }
    }

    !separator.is_some_and(|(allow_trailing, _)| !allow_trailing && has_trailing_sep)
        && (allow_empty || successes)
}

/// Apply a parser until it fails and return the parsed input.
///
/// ### Arguments
/// * `p` - the parser
/// * `allow_empty` - whether no parse should be considered successful.
/// * `separator` - the separator to be used between parses. Use the `no_separator`/`separator`
///                 functions to construct this parameter.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::{many, separator};
/// use anpa::number::integer;
/// use anpa::parsers::skip;
///
/// let parse_nums = many(
///     integer().map(|n: u32| n),
///     false,
///     separator(skip(','), false));
///
/// let input = "1,2,3";
///
/// assert_eq!(parse(parse_nums, input).result, Some("1,2,3"));
/// ```
#[inline]
pub fn many<I: SliceLike, O, O2, S>(p: impl Parser<I, O, S>,
                                    allow_empty: bool,
                                    separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let old_input = s.input;
        many_internal(s, p, |_| {}, allow_empty, separator)
            .then_some(old_input.slice_to(old_input.slice_len() - s.input.slice_len()))
    })
}

/// Apply a parser repeatedly and accumulate a result in the spirit of fold.
///
/// ### Arguments
/// * `p` - the parser
/// * `init` - a function producing the initial result
/// * `f` - a function taking the accumulator as `&mut` along with the result of each
///         successful parse
/// * `allow_empty` - whether no parse should be considered successful.
/// * `separator` - the separator to be used between parses. Use the `no_separator`/`separator`
///                 functions to construct this parameter.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::{fold, separator};
/// use anpa::number::integer;
/// use anpa::parsers::skip;
///
/// let parse_nums = fold(
///     integer().map(|n: u32| n),
///     || 0,
///     |acc, n: u32| *acc += n,
///     false,
///     separator(skip(','), false));
///
/// let input = "1,2,3";
///
/// assert_eq!(parse(parse_nums, input).result, Some(6));
/// ```
#[inline]
pub fn fold<I: SliceLike, O, O2, S, R>(p: impl Parser<I, O, S>,
                                       init: impl FnOnce() -> R + Copy,
                                       f: impl FnOnce(&mut R, O) + Copy,
                                       allow_empty: bool,
                                       separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, R, S> {
    create_parser!(s, {
        let mut res = init();
        many_internal(s, p, |x| f(&mut res, x), allow_empty, separator)
            .then_some(res)
    })
}

#[cfg(feature = "std")]
/// Apply a parser until it fails and store the results in a `Vec`.
///
/// ### Arguments
/// * `p` - the parser
/// * `allow_empty` - whether no parse should be considered successful.
/// * `separator` - the separator to be used between parses. Use the `no_separator`/`separator`
///                 functions to construct this parameter.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::{many_to_vec, separator};
/// use anpa::number::integer;
/// use anpa::parsers::skip;
///
/// let parse_nums = many_to_vec(
///     integer(),
///     false,
///     separator(skip(','), false));
///
/// let input = "1,2,3";
///
/// assert_eq!(parse(parse_nums, input).result, Some(vec![1,2,3]));
/// ```
#[inline]
pub fn many_to_vec<I: SliceLike, O, O2, S>(p: impl Parser<I, O, S>,
                                           allow_empty: bool,
                                           separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, Vec<O>, S> {
    fold(p, Vec::new, |v, x| v.push(x), allow_empty, separator)
}

#[cfg(feature = "std")]
/// Apply a parser until it fails and store the results in a `HashMap`.
/// The parser `p` must have a result type `(K, V)`, where the key `K: Hash + Eq`.
///
/// ### Arguments
/// * `p` - the parser
/// * `allow_empty` - whether no parse should be considered successful.
/// * `separator` - the separator to be used between parses. Use the `no_separator`/`separator`
///                 functions to construct this parameter.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::{many_to_map, right, separator};
/// use anpa::number::{float, integer};
/// use anpa::parsers::{item_while,skip};
/// use anpa::tuplify;
/// use std::collections::HashMap;
///
/// let parse_nums = many_to_map(
///     tuplify!(integer(), right(skip(':'), item_while(|c: char| c.is_alphanumeric()))),
///     false,
///     separator(skip(','), false));
///
/// let input = "1:one,2:two,3:three";
///
/// let expected = HashMap::from([
///     (1, "one"),
///     (2, "two"),
///     (3, "three")
/// ]);
///
/// assert_eq!(parse(parse_nums, input).result, Some(expected));
/// ```
#[inline]
pub fn many_to_map<I: SliceLike, K: Hash + Eq, V, O2, S>(p: impl Parser<I, (K, V), S>,
                                                         allow_empty: bool,
                                                         separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, HashMap<K, V>, S> {
    fold(p, HashMap::new, |m, (k, v)| { m.insert(k, v); }, allow_empty, separator)
}

#[cfg(feature = "std")]
/// Apply a parser until it fails and store the results in a `BTreeMap`.
/// The parser `p` must have a result type `(K, V)`, where the key `K: Ord`.
/// This might give better performance than `many_to_map`.
///
/// ### Arguments
/// * `p` - the parser
/// * `allow_empty` - whether no parse should be considered successful.
/// * `separator` - the separator to be used between parses. Use the `no_separator`/`separator`
///                 functions to construct this parameter.
///
/// ### Example
/// See [`many_to_map`]
#[inline]
pub fn many_to_map_ordered<I: SliceLike, K: Ord, V, O2, S>(p: impl Parser<I, (K, V), S>,
                                                           allow_empty: bool,
                                                           separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, BTreeMap<K, V>, S> {
    fold(p, BTreeMap::new, |m, (k, v)| { m.insert(k, v); }, allow_empty, separator)
}

/// Combine two parsers into a parser that returns the result of the parser
/// that consumed the most input.
///
/// If both parsers consume the same amount of input, the result of the first
/// parser will be chosen.
///
/// If only one of the parsers succeeds, its result will be returned regardless
/// of the consumed size of either parser.
///
/// ### Arguments
/// * `p1` - the first parser
/// * `p2` - the second parser
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::combinators::greedy_or;
/// use anpa::number::integer;
/// use anpa::parsers::item_while;
///
/// let parse_greedy = greedy_or(
///     integer(),
///     item_while(|c: char| c.is_alphanumeric()).map(|_| 0)
/// );
///
/// let input1 = "123";
/// let input2 = "123abc";
///
/// assert_eq!(parse(parse_greedy, input1).result, Some(123));
/// assert_eq!(parse(parse_greedy, input2).result, Some(0));
/// ```
#[inline]
pub fn greedy_or<I: SliceLike, S, O>(p1: impl Parser<I, O, S>,
                                     p2: impl Parser<I, O, S>
) ->  impl Parser<I, O, S> {
    create_parser!(s, {
        let pos = s.input;
        let res1 = p1(s);
        let p1_consumed = pos.slice_len() - s.input.slice_len();
        let p1_pos = s.input;

        s.input = pos;
        let res2 = p2(s);
        let p2_consumed = pos.slice_len() - s.input.slice_len();
        let p1_is_some = res1.is_some();

        let choose_p1 = if p1_is_some == res2.is_some() {
            // Both parsers failed or succeeded. Choose p1 if it consumed the most.
            p1_consumed >= p2_consumed
        }
        else {
            // Otherwise choose p1 if it succeeded.
            p1_is_some
        };

        if choose_p1 {
            s.input = p1_pos;
            res1
        } else {
            res2
        }
    })
}

/// (Description inspired by Parsec's `chainl1`)
///
/// Chain one or more `p` separated by `op`.
/// `op` is a parser that returns a binary function taking as arguments the
/// return type of `p`.
///
/// This parser will return the repeated left associative application of
/// the function returned by `op` applied to the results of `p`.
///
/// This parser can be used to eliminate left recursion, for example in expression
/// grammars.
///
/// ### Arguments
/// * `p` - a parser for arguments to the function parsed by `op`.
/// * `op` - a parser for a binary function.
///
/// ### Example
/// ```
/// use anpa::core::*;
/// use anpa::defer_parser;
/// use anpa::combinators::{chain, or, middle};
/// use anpa::number::integer_signed;
/// use anpa::parsers::{skip, take};
///
/// // A parser that calculates an arihmetic expression.
/// // Whitespace is not supported for simplicity.
/// fn expr<'a>() -> impl StrParser<'a, i64> {
///     let ops = |c| {
///         move |a, b| {
///             match c {
///                 '+' => a + b,
///                 '-' => a - b,
///                 '*' => a * b,
///                 '/' => a / b,
///                 _   => unreachable!()
///             }
///         }
///     };
///
///     let add_op = or(take('+'), take('-')).map(ops);
///     let mul_op = or(take('*'), take('/')).map(ops);
///
///     let atom = or(integer_signed(),
///                   middle(skip('('), defer_parser!(expr()), skip(')')));
///     let factor = chain(atom, mul_op);
///     chain(factor, add_op)
/// }
///
/// let input1 = "-12";
/// let input2 = "12";
/// let input3 = "12+24";
/// let input4 = "2*12+24";
/// let input5 = "(24/3-2-(3+2)-4*3/2)*5";
///
/// assert_eq!(parse(expr(), input1).result, Some(-12));
/// assert_eq!(parse(expr(), input2).result, Some(12));
/// assert_eq!(parse(expr(), input3).result, Some(36));
/// assert_eq!(parse(expr(), input4).result, Some(48));
/// assert_eq!(parse(expr(), input5).result, Some(-25));
/// ```
#[inline]
pub fn chain<I: SliceLike, S, O, F>(p: impl Parser<I, O, S>,
                                    op: impl Parser<I, F, S>
) ->  impl Parser<I, O, S> where F: FnOnce(O, O) -> O {
    create_parser!(s, {
        let mut res = p(s)?;
        loop {
            if let Some(op_f) = op(s) {
                if let Some(res2) = p(s) {
                    res = op_f(res, res2);
                    continue;
                }
            }

            return Some(res)
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{combinators::{greedy_or, many, middle, no_separator, not_empty, times}, core::*, number::integer, parsers::{take, empty, item_while}};

    use super::{fold, or, left};

    fn num_parser() -> impl StrParser<'static, u32> {
        let num = integer();
        or(left(num, take(',')), left(num, empty()))
    }

    #[cfg(feature = "std")]
    #[test]
    fn many_nums_vec() {
        use std::vec;
        use crate::combinators::many_to_vec;
        let p = many_to_vec(num_parser(), true, no_separator());
        let res = parse(p, "1,2,3,4").result.unwrap();
        assert_eq!(res, vec![1,2,3,4]);

        let res = parse(p, "").result.unwrap();
        assert_eq!(res, vec![]);

        let p = many_to_vec(num_parser(), false, no_separator());
        let res = parse(p, "").result;
        assert!(res.is_none());
    }

    #[test]
    fn many_nums() {
        let p = many(num_parser(), true, no_separator());
        let res = parse(p, "1,2,3,4").result.unwrap();
        assert_eq!(res, "1,2,3,4");

        let res = parse(p, "").result.unwrap();
        assert_eq!(res, "");

        let p = many(num_parser(), false, no_separator());
        let res = parse(p, "").result;
        assert!(res.is_none());
    }

    #[test]
    fn fold_add() {
        let p = fold(num_parser(), || 0, |acc, x| *acc += x, false, no_separator());
        let res = parse(p, "1,2,3,4,").result.unwrap();
        assert_eq!(res, 10);
    }

    #[test]
    fn times_test() {
        let p = times(4, left(take('1'), take('2')));
        let res = parse(p, "12121212End");
        assert_eq!(res.result.unwrap(), "12121212");
        assert_eq!(res.state, "End");

        let res = parse(p, "121212").result;
        assert!(res.is_none());
    }

    #[test]
    fn recursive_parens() {
        fn in_parens<'a>() -> impl StrParser<'a> {
            defer_parser!(or(not_empty(item_while(|c: char| c.is_alphanumeric())), middle(take('('), in_parens(), take(')'))))
        }

        let x = "(((((((((sought)))))))))";

        let res = parse(in_parens(), x);
        assert_eq!(res.result.unwrap(), "sought");
        assert!(res.state.is_empty());
    }

    #[test]
    fn greedy_or_test() {
        let x = "12344a";

        let digit_parser = item_while(|c: char| c.is_ascii_digit());
        let seq_parser = take("1234");

        let greedy_parser = greedy_or(seq_parser, digit_parser);

        let res = parse(greedy_parser, x);
        assert_eq!(res.result.unwrap(), "12344");
        assert_eq!(res.state, "a");

        let smaller_seq_parser = take("123");

        let full_parser = digit_parser.right(take("a"));

        let greedy_parser = greedy_or!(smaller_seq_parser, full_parser, seq_parser);
        let res = parse(greedy_parser, x);
        assert_eq!(res.result.unwrap(), "a");
        assert!(res.state.is_empty());
    }
}