use std::vec;

use crate::{slicelike::SliceLike, core::{Parser, AnpaState}};

pub fn bind<I: SliceLike, O1, O2, P, S>(p: impl Parser<I, O1, S>, 
                                        f: impl FnOnce(O1) -> P + Copy
) -> impl Parser<I, O2, S> where P: Parser<I, O2, S> {
    create_parser!(s, f(p(s)?)(s))
}

pub fn not_empty<I: SliceLike, O: SliceLike, S>(p: impl Parser<I, O, S>) -> impl Parser<I, O, S> {
    create_parser!(s, p(s).filter(|x| !x.slice_is_empty()))
}

pub fn right<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                      p2: impl Parser<I, O2, S>
) ->  impl Parser<I, O2, S> {
    create_parser!(s, {
        p1(s)?;
        p2(s)
    })
}

pub fn left<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
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

pub fn or<I: SliceLike, O, S>(p1: impl Parser<I, O, S>,
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

pub fn or_diff<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
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

pub fn lift_to_state<I: SliceLike, S, O1, O2>(f: impl FnOnce(&mut S, O1) -> O2 + Copy,
                                              p: impl Parser<I, O1, S>
) -> impl Parser<I, O2, S> {
    create_parser!(s, {
        let res = p(s)?;
        Some(f(&mut s.user_state, res))
    })
}

fn many_internal<const ALLOW_EMPTY: bool, I: SliceLike, O, S>(
    s: &mut AnpaState<I, S>,
    p: impl Parser<I, O, S>,
    mut f: impl FnMut(O) -> ()
) -> Option<()> {
    let mut i = 0;
    loop {
        let Some(res) = p(s) else {
            break;
        };
        i += 1;
        f(res);
    }
    (ALLOW_EMPTY || (i > 0)).then_some(())
}

pub fn many0<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, Vec<O>, S> {
    create_parser!(s, {
        let mut vec = vec![];
        many_internal::<true,_,_,_>(s, p, |x| vec.push(x)).map(move |_| vec)
    })
}

pub fn many1<I: SliceLike, O, S>(p: impl Parser<I, O, S>) -> impl Parser<I, Vec<O>, S> {
    create_parser!(s, {
        let mut vec = vec![];
        many_internal::<false,_,_,_>(s, p, |x| vec.push(x)).map(move |_| vec)
    })
}

pub fn fold<T: Copy, I: SliceLike, O, S>(acc: T,
                                         p: impl Parser<I, O, S>,
                                         f: impl Fn(&mut T, O) -> () + Copy
) -> impl Parser<I, T, S> {
    create_parser!(s, {
        let mut acc = acc;
        many_internal::<false,_,_,_>(s, p, |x| {
            f(&mut acc, x)
        }).map(move |_| acc)
    })
}