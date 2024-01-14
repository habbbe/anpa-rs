use crate::combinators::{bind, right, left, filter, into_type};

pub struct AnpaState<'a, T, S> {
    pub input: T,
    pub user_state: &'a mut S,
}

pub trait Parser<I, O, S>: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy {}
impl<I, O, S, F: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy> Parser<I, O, S> for F {}

pub trait ParserExt<I, O, S>: Parser<I, O, S> {
    fn into_type<T: From<O>>(self) -> impl Parser<I, T, S>;
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S>;
    fn filter(self, f: impl FnOnce(&O) -> bool + Copy) -> impl Parser<I, O, S>;
    fn bind<O2, P: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P + Copy) -> impl Parser<I, O2, S>;
    fn right<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O2, S>;
    fn left<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O, S>;
    fn debug(self, name: &'static str) -> impl Parser<I, O, S>;
}

impl<I, O, S, P: Parser<I, O, S>> ParserExt<I, O ,S> for P {
    #[inline]
    fn into_type<T: From<O>>(self) -> impl Parser<I, T, S> {
        into_type(self)
    }

    #[inline]
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S> {
        lift!(f, self)
    }

    #[inline]
    fn filter(self, f: impl FnOnce(&O) -> bool + Copy) -> impl Parser<I, O, S> {
        filter(self, f)
    }

    #[inline]
    fn bind<O2, P2: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P2 + Copy) -> impl Parser<I, O2, S> {
        bind(self, f)
    }

    #[inline]
    fn right<O2, P2: Parser<I, O2, S>>(self, p: P2) -> impl Parser<I, O2, S> {
        right(self, p)
    }

    #[inline]
    fn left<O2, P2: Parser<I, O2, S>>(self, p: P2) -> impl Parser<I, O, S> {
        left(self, p)
    }

    fn debug(self, name: &'static str) -> impl Parser<I, O, S> {
        create_parser!(s, {
            let res = self(s);
            match res {
                Some(_) => println!("{}: Succeeded", name),
                None => println!("{}: Failed", name),
            }
            res
        })
    }
}

pub fn parse_state<I, O, S>(p: impl Parser<I, O, S>,
                                       input: I,
                                       user_state: &mut S) -> (AnpaState<I, S>, Option<O>) {
    let mut parser_state = AnpaState { input, user_state };
    let result = p(&mut parser_state);
    (parser_state, result)
}

pub fn parse<I, O>(p: impl Parser<I, O, ()>,
                   input: I) -> (I, Option<O>) {
    let mut parser_state = AnpaState { input, user_state: &mut () };
    let result = p(&mut parser_state);
    (parser_state.input, result)
}