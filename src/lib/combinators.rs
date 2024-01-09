use std::{collections::{HashMap, BTreeMap}, hash::Hash};

use crate::{slicelike::SliceLike, core::{Parser, AnpaState}, parsers::success};

pub fn bind<I, O1, O2, P, S>(p: impl Parser<I, O1, S>,
                             f: impl FnOnce(O1) -> P + Copy
) -> impl Parser<I, O2, S> where P: Parser<I, O2, S> {
    create_parser!(s, f(p(s)?)(s))
}

pub fn succeed<I, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, Option<O>, S> {
    create_parser!(s, {
        Some(p(s))
    })
}

pub fn not_empty<I, O: SliceLike, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    create_parser!(s, p(s).filter(|x| !x.slice_is_empty()))
}

pub fn count_consumed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (usize, O), S> {
    create_parser!(s, {
        let old = s.input.slice_len();
        let res = p(s)?;
        let count = old - s.input.slice_len();
        Some((count, res))
    })
}

pub fn and_parsed<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, (I, O), S> {
    create_parser!(s, {
        let old_input = s.input;
        let res = p(s)?;
        Some((old_input.slice_to(old_input.slice_len() - s.input.slice_len()), res))
    })
}

pub fn times<I: SliceLike, O, S>(times: u32, p: impl Parser<I, O, S>) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let old_input = s.input;
        for _ in 0..times {
            p(s)?;
        }
        Some(old_input.slice_to(old_input.slice_len() - s.input.slice_len()))
    })
}

pub fn right<I, S, O1, O2>(p1: impl Parser<I, O1, S>,
                           p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O2, S> {
    create_parser!(s, {
        p1(s)?;
        p2(s)
    })
}

pub fn left<I, S, O1, O2>(p1: impl Parser<I, O1, S>,
                          p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O1, S> {
    create_parser!(s, {
        if let a@Some(_) = p1(s) {
            p2(s)?;
            a
        } else {
            None
        }
    })
}

pub fn middle<I, S, O1, O2, O3>(p1: impl Parser<I, O1, S>,
                                p2: impl Parser<I, O2, S>,
                                p3: impl Parser<I, O3, S>
) ->  impl Parser<I, O2, S> {
    right(p1, left(p2, p3))
}

pub fn or<I: Copy, O, S>(p1: impl Parser<I, O, S>,
                         p2: impl Parser<I, O, S>
) -> impl Parser<I, O, S> {
    create_parser!(s, {
        let pos = s.input;
        if let a@Some(_) = p1(s) {
            a
        } else {
            s.input = pos;
            p2(s)
        }
    })
}

pub fn or_diff<I: Copy, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                   p2: impl Parser<I, O2, S>
) -> impl Parser<I, (), S> {
    create_parser!(s, {
        let pos = s.input;
        if p1(s).is_some() {
            Some(())
        } else {
            s.input = pos;
            p2(s)?;
            Some(())
        }
    })
}

pub fn lift_to_state<I, S, O1, O2>(f: impl FnOnce(&mut S, O1) -> O2 + Copy,
                                   p: impl Parser<I, O1, S>
) -> impl Parser<I, O2, S> {
    create_parser!(s, {
        let res = p(s)?;
        Some(f(&mut s.user_state, res))
    })
}

pub fn no_separator<I, S>() -> Option<(bool, impl Parser<I, (), S>)> {
    Some((false, success())).filter(|_| false)
}

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
            if let Some((false, _)) = separator {
                if successes {
                    return None;
                }
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

pub fn many<I: SliceLike, O, S>(p: impl Parser<I, O, S>,
                                    allow_empty: bool,
                                    separator: Option<(bool, impl Parser<I, (), S>)>,
) -> impl Parser<I, I, S> {
    create_parser!(s, {
        let old_input = s.input;
        many_internal(s, p, |_| {}, allow_empty, separator)?;
        let res = old_input.slice_to(old_input.slice_len() - s.input.slice_len());
        Some(res)
    })
}

pub fn many_to_vec<I, O, O2, S>(p: impl Parser<I, O, S>,
                            allow_empty: bool,
                            separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, Vec<O>, S> {
    create_parser!(s, {
        let mut vec = vec![];
        many_internal(s, p, |x| vec.push(x), allow_empty, separator).map(move |_| vec)
    })
}

pub fn many_to_map<I, K: Hash + Eq, V, O2, S>(p: impl Parser<I, (K, V), S>,
                                          allow_empty: bool,
                                          separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, HashMap<K, V>, S> {
    create_parser!(s, {
        let mut map = HashMap::new();
        many_internal(s, p, |(k, v)| {map.insert(k, v);}, allow_empty, separator).map(move |_| map)
    })
}

pub fn many_to_map_ordered<I, K: Ord, V, O2, S>(p: impl Parser<I, (K, V), S>,
                                          allow_empty: bool,
                                          separator: Option<(bool, impl Parser<I, O2, S>)>,
) -> impl Parser<I, BTreeMap<K, V>, S> {
    create_parser!(s, {
        let mut map = BTreeMap::new();
        many_internal(s, p, |(k, v)| {map.insert(k, v);}, allow_empty, separator).map(move |_| map)
    })
}

pub fn fold<T: Copy, I, O, S, P: Parser<I, O, S>>(acc: T,
                              p: P,
                              f: impl Fn(&mut T, O) -> () + Copy
) -> impl Parser<I, T, S> {
    create_parser!(s, {
        let mut acc = acc;
        many_internal(s, p, |x| {
            f(&mut acc, x)
        }, true, no_separator()).map(move |_| acc)
    })
}

#[cfg(test)]
mod tests {
    use crate::{parsers::{item, empty, integer_u32, item_while}, core::{*}, combinators::{times, middle, many_to_vec, many, no_separator}};

    use super::{fold, or, left};

    fn num_parser() -> impl Parser<&'static str, u32, ()> {
        let num = integer_u32(10);
        or(left(num, item(',')), left(num, empty()))
    }

    #[test]
    fn many_nums_vec() {
        let p = many_to_vec(num_parser(), true, no_separator());
        let res = parse(p, "1,2,3,4").1.unwrap();
        assert_eq!(res, vec![1,2,3,4]);

        let res = parse(p, "").1.unwrap();
        assert_eq!(res, vec![]);

        let p = many_to_vec(num_parser(), false, no_separator());
        let res = parse(p, "").1;
        assert!(res.is_none());
    }

    #[test]
    fn many_nums() {
        let p = many(num_parser(), true, no_separator());
        let res = parse(p, "1,2,3,4").1.unwrap();
        assert_eq!(res, "1,2,3,4");

        let res = parse(p, "").1.unwrap();
        assert_eq!(res, "");

        let p = many(num_parser(), false, no_separator());
        let res = parse(p, "").1;
        assert!(res.is_none());
    }

    #[test]
    fn fold_add() {
        let p = fold(0, num_parser(), |acc, x| *acc += x);
        let res = parse(p, "1,2,3,4,").1.unwrap();
        assert_eq!(res, 10);
    }

    #[test]
    fn times_test() {
        let p = times(4, left(item('1'), item('2')));
        let res = parse(p, "12121212End");
        assert_eq!(res.1.unwrap(), "12121212");
        assert_eq!(res.0, "End");

        let res = parse(p, "121212").1;
        assert!(res.is_none());
    }

    #[test]
    fn recursive_parens() {
        fn in_parens<'a, S>() -> impl Parser<&'a str, &'a str, S> {
            defer_parser!(or(item_while(|c: char| c.is_alphanumeric()), middle(item('('), in_parens(), item(')'))))
        }

        let x = "(((((((((sought)))))))))";

        let res = parse(in_parens(), x);
        assert_eq!(res.1.unwrap(), "sought");
        assert!(res.0.is_empty());
    }
}