#[cfg(feature = "std")]
use std::{collections::{BTreeMap, HashMap}, vec::Vec, hash::Hash};

use crate::{slicelike::SliceLike, core::{Parser, AnpaState}, parsers::success};

/// Create a new parser by taking the result of `p`, and applying `f`.
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the function to generate the new parser
#[inline]
pub fn bind<I, O1, O2, P, S>(p: impl Parser<I, O1, S>,
                             f: impl FnOnce(O1) -> P + Copy
) -> impl Parser<I, O2, S> where P: Parser<I, O2, S> {
    create_parser!(s, f(p(s)?)(s))
}

/// Create a new parser by applying a transformation `f` to the result of `p`.
/// This differs from `bind` in that the transformation does not return a new
/// parser. Use this combinator if you just want to modify the result.
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the transformation function.
#[inline]
pub fn map<I, O, O2, S>(p: impl Parser<I, O, S>,
                        f: impl FnOnce(O) -> O2 + Copy
) -> impl Parser<I, O2, S> {
    lift!(f, p)
}

/// Transform the parser `p` into a parser with a different result by means of `Into`.
/// The existing type must implement `Into<T>` for the requested type `T`.
///
/// ### Arguments
/// * `p` - the parser
#[inline]
pub fn into_type<I, O: Into<T>, T, S>(p: impl Parser<I, O, S>) -> impl Parser<I, T, S> {
    map(p, O::into)
}

/// Accept or reject the parse based on the predicate `f`.
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the predicate
#[inline]
pub fn filter<I, O, S>(p: impl Parser<I, O, S>,
                       f: impl FnOnce(&O) -> bool + Copy
) -> impl Parser<I, O, S> {
    create_parser!(s, p(s).filter(f))
}

/// Create a new parser by applying a transformation `f` to the result of `p`.
/// Unlike `map`, this combinator allows optional rejection of the parse by returning
/// `Some` or `None` in the transformation.
///
/// ### Arguments
/// * `p` - the parser
/// * `f` - the transformation function.
#[inline]
pub fn map_if<I, O, O2, S>(p: impl Parser<I, O, S>,
                           f: impl FnOnce(O) -> Option<O2> + Copy
) -> impl Parser<I, O2, S> {
    create_parser!(s, {
        p(s).and_then(f)
    })
}

/// Transform a parser to a parser that always succeeds. The resulting parser will
/// have its result type changed to `Option`, to allow for introspection of the result
/// of the parse.
///
/// ### Arguments
/// * `p` - the parser
#[inline]
pub fn succeed<I, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, Option<O>, S> {
    create_parser!(s, {
        Some(p(s))
    })
}

/// Transform a parser to a parser that does not consume any input.
///
/// ### Arguments
/// * `p` - the parser
#[inline]
pub fn peek<I: Copy, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
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
#[inline]
pub fn not_empty<I, O: SliceLike, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    filter(p, |r| !r.slice_is_empty())
}

/// Transform a parser to a parser that does not consume any input on failure.
///
/// ### Arguments
/// * `p` - the parser
#[inline]
pub fn attempt<I: Copy, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
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
/// ### Arguments
/// * `p` - the parser
#[inline]
pub fn count_consumed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (usize, O), S> {
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
/// ### Arguments
/// * `p1` - the first parser (result will be ignored)
/// * `p2` - the second parser
#[inline]
pub fn right<I, S, O1, O2>(p1: impl Parser<I, O1, S>,
                           p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O2, S> {
    create_parser!(s, {
        p1(s).and_then(|_| p2(s))
    })
}

/// Combine one parser with another, while ignoring the result of the latter.
/// The second parser will only be attempted if the first succeeds.
///
/// ### Arguments
/// * `p1` - the first parser
/// * `p2` - the second parser (result will be ignored)
#[inline]
pub fn left<I, S, O1, O2>(p1: impl Parser<I, O1, S>,
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
#[inline]
pub fn middle<I, S, O1, O2, O3>(p1: impl Parser<I, O1, S>,
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
        /// $comment
        ///
        /// ### Arguments
        /// * `p1` - the first parser
        /// * `p2` - the second parser
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
        /// $comment
        ///
        /// ### Arguments
        /// * `p1` - the first parser
        /// * `p2` - the second parser
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
#[inline]
pub fn lift_to_state<I, S, O1, O2>(f: impl FnOnce(&mut S, O1) -> O2 + Copy,
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
pub fn no_separator<I, S>() -> Option<(bool, impl Parser<I, (), S>)> {
    return None;

    // Unreachable, but provides type/size information about the return value
    Some((false, success()))
}

#[inline(always)]
fn many_internal<I, O, O2, S>(
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
#[inline]
pub fn fold<I, O, O2, S, R>(p: impl Parser<I, O, S>,
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
#[inline]
pub fn many_to_vec<I, O, O2, S>(p: impl Parser<I, O, S>,
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
#[inline]
pub fn many_to_map<I, K: Hash + Eq, V, O2, S>(p: impl Parser<I, (K, V), S>,
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
#[inline]
pub fn many_to_map_ordered<I, K: Ord, V, O2, S>(p: impl Parser<I, (K, V), S>,
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

#[cfg(test)]
mod tests {
    use crate::{combinators::{greedy_or, many, middle, no_separator, not_empty, times}, core::*, number::integer, parsers::{elem, empty, item_while}};

    use super::{fold, or, left};

    fn num_parser() -> impl StrParser<'static, u32> {
        let num = integer();
        or(left(num, elem(',')), left(num, empty()))
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
        let p = times(4, left(elem('1'), elem('2')));
        let res = parse(p, "12121212End");
        assert_eq!(res.result.unwrap(), "12121212");
        assert_eq!(res.state, "End");

        let res = parse(p, "121212").result;
        assert!(res.is_none());
    }

    #[test]
    fn recursive_parens() {
        fn in_parens<'a>() -> impl StrParser<'a> {
            defer_parser!(or(not_empty(item_while(|c: char| c.is_alphanumeric())), middle(elem('('), in_parens(), elem(')'))))
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
        let seq_parser = elem("1234");

        let greedy_parser = greedy_or(seq_parser, digit_parser);

        let res = parse(greedy_parser, x);
        assert_eq!(res.result.unwrap(), "12344");
        assert_eq!(res.state, "a");

        let smaller_seq_parser = elem("123");

        let full_parser = digit_parser.right(elem("a"));

        let greedy_parser = greedy_or!(smaller_seq_parser, full_parser, seq_parser);
        let res = parse(greedy_parser, x);
        assert_eq!(res.result.unwrap(), "a");
        assert!(res.state.is_empty());
    }
}