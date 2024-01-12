use crate::{slicelike::SliceLike, combinators::{bind, right, left}};

pub struct AnpaState<'a, T, S> {
    pub input: T,
    pub user_state: &'a mut S,
}

pub trait Parser<I, O, S>: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy {}
impl<I, O, S, F: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy> Parser<I, O, S> for F {}

pub trait ParserExt<I, O, S>: Parser<I, O, S> {
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S>;
    fn bind<O2, P: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P + Copy) -> impl Parser<I, O2, S>;
    fn right<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O2, S>;
    fn left<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O, S>;
    fn debug(self, name: &'static str) -> impl Parser<I, O, S>;
}

impl<I: SliceLike, O, S, P: Parser<I, O, S>> ParserExt<I, O ,S> for P {
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S> {
        lift!(f, self)
    }

    fn bind<O2, P2: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P2 + Copy) -> impl Parser<I, O2, S> {
        bind(self, f)
    }

    fn right<O2, P2: Parser<I, O2, S>>(self, p: P2) -> impl Parser<I, O2, S> {
        right(self, p)
    }

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