use std::{collections::{HashMap, BTreeMap}, hash::Hash};

use crate::{slicelike::SliceLike, core::{Parser, AnpaState}, parsers::success};

#[inline]
pub fn bind<I, O1, O2, P, S>(p: impl Parser<I, O1, S>,
                             f: impl FnOnce(O1) -> P + Copy
) -> impl Parser<I, O2, S> where P: Parser<I, O2, S> {
    create_parser!(s, f(p(s)?)(s))
}

#[inline]
pub fn into_type<I, O, T: From<O>, S>(p: impl Parser<I, O, S>) -> impl Parser<I, T, S> {
    lift!(|x: O| x.into(), p)
}

#[inline]
pub fn filter<I, O, S>(p: impl Parser<I, O, S>,
                       f: impl FnOnce(&O) -> bool + Copy
) -> impl Parser<I, O, S> {
    create_parser!(s, p(s).filter(f))
}

#[inline]
pub fn map_if<I, O, O2, S>(p: impl Parser<I, O, S>,
                           f: impl FnOnce(O) -> Option<O2> + Copy
) -> impl Parser<I, O2, S> {
    create_parser!(s, {
        p(s).and_then(f)
    })
}

#[inline]
pub fn succeed<I, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, Option<O>, S> {
    create_parser!(s, {
        Some(p(s))
    })
}

#[inline]
pub fn peek<I: Copy, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    create_parser!(s, {
        let pos = s.input;
        let res = p(s);
        s.input = pos;
        res
    })
}

#[inline]
pub fn not_empty<I, O: SliceLike, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    filter(p, |r| !r.slice_is_empty())
}

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

#[inline]
pub fn count_consumed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (usize, O), S> {
    create_parser!(s, {
        let old = s.input.slice_len();
        let res = p(s)?;
        let count = old - s.input.slice_len();
        Some((count, res))
    })
}

#[inline]
pub fn and_parsed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (I, O), S> {
    create_parser!(s, {
        let old_input = s.input;
        let res = p(s)?;
        Some((old_input.slice_to(old_input.slice_len() - s.input.slice_len()), res))
    })
}

#[inline]
pub fn get_parsed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let old_input = s.input;
        p(s)?;
        Some(old_input.slice_to(old_input.slice_len() - s.input.slice_len()))
    })
}

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

#[inline]
pub fn right<I, S, O1, O2>(p1: impl Parser<I, O1, S>,
                           p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O2, S> {
    create_parser!(s, {
        p1(s).and_then(|_| p2(s))
    })
}

#[inline]
pub fn left<I, S, O1, O2>(p1: impl Parser<I, O1, S>,
                          p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O1, S> {
    create_parser!(s, {
        p1(s).and_then(|res| p2(s).map(|_| res))
    })
}

#[inline]
pub fn middle<I, S, O1, O2, O3>(p1: impl Parser<I, O1, S>,
                                p2: impl Parser<I, O2, S>,
                                p3: impl Parser<I, O3, S>
) ->  impl Parser<I, O2, S> {
    right(p1, left(p2, p3))
}

macro_rules! internal_or {
    ($id:ident, $allow_partial:expr) => {
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

internal_or!(or, true);
internal_or!(or_no_partial, false);

macro_rules! internal_or_diff {
    ($id:ident, $allow_partial:expr) => {
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

internal_or_diff!(or_diff, true);
internal_or_diff!(or_diff_no_partial, false);

#[inline]
pub fn lift_to_state<I, S, O1, O2>(f: impl FnOnce(&mut S, O1) -> O2 + Copy,
                                   p: impl Parser<I, O1, S>
) -> impl Parser<I, O2, S> {
    create_parser!(s, {
        p(s).map(|res| f(&mut s.user_state, res))
    })
}

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

        if let Some((_, sep)) = separator {
            if sep(s).is_none() {
                break;
            }
        }
    }
    (allow_empty || successes).then_some(())
}

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

    fn num_parser() -> impl Parser<&'static str, u32, ()> {
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
        fn in_parens<'a, S>() -> impl Parser<&'a str, &'a str, S> {
            defer_parser!(or(item_while(|c: char| c.is_alphanumeric()), middle(item('('), in_parens(), item(')'))))
        }

        let x = "(((((((((sought)))))))))";

        let res = parse(in_parens(), x);
        assert_eq!(res.result.unwrap(), "sought");
        assert!(res.state.is_empty());
    }
}