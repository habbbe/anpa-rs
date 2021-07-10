pub struct State<'a, T, I: Iterator<Item=T>, S> {
    pub iterator: I,
    pub user_state: &'a mut S
}

pub struct StateVal<T, I: Iterator<Item=T>, S> {
    pub iterator: I,
    pub user_state: S
}

pub trait Parser<T, I: Iterator<Item=T>, S, R>: FnOnce(&mut State<T, I, S>) -> Option<R> {}
impl<T, I: Iterator<Item=T>, S, R, F> Parser<T, I, S, R> for F where F: FnOnce(&mut State<T, I, S>) -> Option<R> {}

pub fn parse<T, I: Iterator<Item=T>, S, R>(p: impl Parser<T, I, S, R>, i: I, state: &mut S) -> (State<T, I, S>, Option<R>) {
    let mut parser_state = State {iterator: i, user_state: state};
    let result = p(&mut parser_state);
    (parser_state, result)
}

