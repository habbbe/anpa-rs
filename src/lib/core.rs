use crate::combinators::{bind, right, left, filter, into_type, map_if, map};

pub struct AnpaState<'a, T, S> {
    pub input: T,
    pub user_state: &'a mut S,
}

pub struct AnpaResult<T, O> {
    pub state: T,
    pub result: Option<O>
}

pub trait Parser<I, O, S>: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy {}
impl<I, O, S, F: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy> Parser<I, O, S> for F {}

pub trait ParserExt<I, O, S>: Parser<I, O, S> {
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S>;
    fn map_if<O2>(self, f: impl FnOnce(O) -> Option<O2> + Copy) -> impl Parser<I, O2, S>;
    fn filter(self, f: impl FnOnce(&O) -> bool + Copy) -> impl Parser<I, O, S>;
    fn bind<O2, P: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P + Copy) -> impl Parser<I, O2, S>;
    fn right<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O2, S>;
    fn left<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O, S>;
    fn debug(self, name: &'static str) -> impl Parser<I, O, S>;
}

/// Trait for parsers with a result that can be converted into another by means of `Into`.
pub trait ParserInto<I, O1: Into<O2>, O2, S>: Parser<I, O1, S> {
    /// Transform this parser into a parser with a different result.
    fn into_type(self) -> impl Parser<I, O2, S>;
}

impl<I, O1: Into<O2>, O2, S, P: Parser<I, O1, S>> ParserInto<I, O1, O2, S> for P {
    #[inline]
    fn into_type(self) -> impl Parser<I, O2, S> {
        into_type(self)
    }
}

impl<I, O, S, P: Parser<I, O, S>> ParserExt<I, O ,S> for P {
    #[inline]
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S> {
        map(self, f)
    }

    #[inline]
    fn map_if<O2>(self, f: impl FnOnce(O) -> Option<O2> + Copy) -> impl Parser<I, O2, S> {
        map_if(self, f)
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
                                       user_state: &mut S) -> AnpaResult<AnpaState<I, S>, O> {
    let mut parser_state = AnpaState { input, user_state };
    let result = p(&mut parser_state);
    AnpaResult { state: parser_state, result }
}

pub fn parse<I, O>(p: impl Parser<I, O, ()>,
                   input: I) -> AnpaResult<I, O> {
    let mut parser_state = AnpaState { input, user_state: &mut () };
    let result = p(&mut parser_state);
    AnpaResult { state: parser_state.input, result }
}