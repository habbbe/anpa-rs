use std::borrow::Borrow;
use std::marker::PhantomData;

pub struct Nothing;

pub struct State<'a, T, X: Borrow<T>, I: Iterator<Item=X>, S> {
    pub iterator: I,
    pub user_state: &'a mut S,
    phantom: PhantomData<T>,
}

pub struct StateVal<T, I: Iterator<Item=T>, S> {
    pub iterator: I,
    pub user_state: S
}

pub trait Parser<T, X: Borrow<T>, I: Iterator<Item=X>, S, R>: FnOnce(&mut State<T, X, I, S>) -> Option<R> {}
impl<T, X: Borrow<T>, I: Iterator<Item=X>, S, R, F> Parser<T, X, I, S, R> for F
    where F: FnOnce(&mut State<T, X, I, S>) -> Option<R> {}

pub fn parse<T, X: Borrow<T>, I: Iterator<Item=X>, S, R>(p: impl Parser<T, X, I, S, R>,
                                                         i: I, state: &mut S)
    -> (State<T, X, I, S>, Option<R>) {
    let mut parser_state = State {iterator: i, user_state: state, phantom: PhantomData};
    let result = p(&mut parser_state);
    (parser_state, result)
}

