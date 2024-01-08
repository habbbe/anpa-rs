use crate::{slicelike::SliceLike, combinators::{bind, self}};

pub struct AnpaState<'a, T, S> {
    pub input: T,
    pub user_state: &'a mut S,
}

pub trait ParserExt<I, O, S>: Parser<I, O, S> {
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S>;
    fn bind<O2, P: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P + Copy) -> impl Parser<I, O2, S>;
}

impl<I, O, S, P: Parser<I, O, S>> ParserExt<I, O ,S> for P {
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S> {
        lift!(f, self)
    }

    fn bind<O2, P2: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P2 + Copy) -> impl Parser<I, O2, S> {
        bind(self, f)
    }
}

pub trait Parser<I, O, S>: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy {}
impl<I, O, S, F: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy> Parser<I, O, S> for F {}

pub fn parse_state<I: SliceLike, O, S>(p: impl Parser<I, O, S>,
                                       input: I,
                                       user_state: &mut S) -> (AnpaState<I, S>, Option<O>) {
    let mut parser_state = AnpaState { input, user_state };
    let result = p(&mut parser_state);
    (parser_state, result)
}

pub fn parse<I: SliceLike, O>(p: impl Parser<I, O, ()>,
                              input: I) -> (I, Option<O>) {
    let mut parser_state = AnpaState { input, user_state: &mut () };
    let result = p(&mut parser_state);
    (parser_state.input, result)
}