use std::{collections::{HashMap, BTreeMap}, hash::Hash};

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
        p(s).map(|res| f(&mut s.user_state, res))
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
#[inline]
pub fn no_separator<I, S>() -> Option<(bool, impl Parser<I, (), S>)> {
    if true {
        None
    } else {
        // Only for type checking
        Some((false, success()))
    }
}

#[inline(always)]
fn many_internal<I, O, O2, S>(
    s: &mut AnpaState<I, S>,
    p: impl Parser<I, O, S>,
    mut f: impl FnMut(O) -> (),
    allow_empty: bool,
    separator: Option<(bool, impl Parser<I, O2, S>)>
) -> Option<()> {
    let mut successes = false;
    loop {
        let Some(res) = p(s) else {
            if let (Some((false, _)), true) = (separator, successes) {
                return None
            }
            break;
        };

        f(res);
        successes = true;

        if separator.is_some_and(|(_, sep)| sep(s).is_none()) {
            break;
        }
    }
    (allow_empty || successes).then_some(())
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
            .map(move |_| old_input.slice_to(old_input.slice_len() - s.input.slice_len()))
    })
}

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
    create_parser!(s, {
        let mut vec = vec![];
        many_internal(s, p, |x| vec.push(x), allow_empty, separator)
            .map(move |_| vec)
    })
}

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
    create_parser!(s, {
        let mut map = HashMap::new();
        many_internal(s, p, |(k, v)| {map.insert(k, v);}, allow_empty, separator)
            .map(move |_| map)
    })
}

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
    create_parser!(s, {
        let mut map = BTreeMap::new();
        many_internal(s, p, |(k, v)| {map.insert(k, v);}, allow_empty, separator)
            .map(move |_| map)
    })
}

/// Apply a parser repeatedly and accumulate a result in the spirit of fold.
///
/// ### Arguments
/// * `acc` - the accumulator
/// * `p` - the parser
/// * `f` - a function taking the accumulator as `&mut` along with the result of each
///         successful parse
#[inline]
pub fn fold<T: Copy, I, O, S, P: Parser<I, O, S>>(acc: T,
                                                  p: P,
                                                  f: impl Fn(&mut T, O) -> () + Copy
) -> impl Parser<I, T, S> {
    create_parser!(s, {
        let mut acc = acc;
        many_internal(s, p, |x| { f(&mut acc, x) }, true, no_separator())
            .map(move |_| acc)
    })
}

#[cfg(test)]
mod tests {
    use crate::{parsers::{item, empty, item_while}, core::{*}, combinators::{times, middle, many_to_vec, many, no_separator}, number::integer};

    use super::{fold, or, left};

    fn num_parser() -> impl StrParser<'static, u32> {
        let num = integer();
        or(left(num, item(',')), left(num, empty()))
    }

    #[test]
    fn many_nums_vec() {
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
        let p = fold(0, num_parser(), |acc, x| *acc += x);
        let res = parse(p, "1,2,3,4,").result.unwrap();
        assert_eq!(res, 10);
    }

    #[test]
    fn times_test() {
        let p = times(4, left(item('1'), item('2')));
        let res = parse(p, "12121212End");
        assert_eq!(res.result.unwrap(), "12121212");
        assert_eq!(res.state, "End");

        let res = parse(p, "121212").result;
        assert!(res.is_none());
    }

    #[test]
    fn recursive_parens() {
        fn in_parens<'a>() -> impl StrParser<'a> {
            defer_parser!(or(item_while(|c: char| c.is_alphanumeric()), middle(item('('), in_parens(), item(')'))))
        }

        let x = "(((((((((sought)))))))))";

        let res = parse(in_parens(), x);
        assert_eq!(res.result.unwrap(), "sought");
        assert!(res.state.is_empty());
    }
}