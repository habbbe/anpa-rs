use crate::combinators::{bind, right, left, filter, into_type, map_if, map};

/// The state being passed around during parsing.
pub struct AnpaState<'a, T, S> {
    /// The current state of the input under parse.
    pub input: T,

    /// The provided user state (if any).
    pub user_state: &'a mut S,
}

/// The final result of a parse.
pub struct AnpaResult<T, O> {
    /// The final state of the parse.
    pub state: T,

    /// The result of the parse.
    pub result: Option<O>
}

/// The base trait for all parsers.
///
/// If the output of a parser is the same as the input (e.g. if the result is the
/// parsed input), the output type parameter `O` can be omitted.
///
/// If no user state is used when parsing, the state type parameter `S` can be omitted.
pub trait Parser<I, O = I, S = ()>: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy {}

// Some convenience "aliases" for common parser types
create_parser_trait!(StrParser, str, "Convenience alias for a parser that parses a `&'a str`.");
create_parser_trait!(U8Parser, [u8], "Convenience alias for a parser that parses a `&'a [u8]`.");

/// All copyable `FnOnce` functions with the correct signature are considered parsers.
impl<I, O, S, F: FnOnce(&mut AnpaState<I, S>) -> Option<O> + Copy> Parser<I, O, S> for F {}

/// Convenince extension functions for all parsers.
pub trait ParserExt<I, O, S>: Parser<I, O, S> {

    /// Transform the result of this parser.
    fn map<O2>(self, f: impl FnOnce(O) -> O2 + Copy) -> impl Parser<I, O2, S>;

    /// Transform the result of this parser, or fail, by returning `Some` or `None` respectively.
    fn map_if<O2>(self, f: impl FnOnce(O) -> Option<O2> + Copy) -> impl Parser<I, O2, S>;

    /// Accept or reject the parse based on the predicate `f`.
    fn filter(self, f: impl FnOnce(&O) -> bool + Copy) -> impl Parser<I, O, S>;

    /// Create a new parser by taking the result of the previous and returning a new parser.
    fn bind<O2, P: Parser<I, O2, S>>(self, f: impl FnOnce(O) -> P + Copy) -> impl Parser<I, O2, S>;

    /// Combine this parser with another, while ignoring the result of the former.
    fn right<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O2, S>;

    /// Combine this parser with another, while ignoring the result of the latter.
    fn left<O2, P: Parser<I, O2, S>>(self, p: P) -> impl Parser<I, O, S>;

    #[cfg(feature = "std")]
    /// Add some simple debug information to this parser.
    fn debug(self, name: &'static str) -> impl Parser<I, O, S>;
}

/// Trait for parsers with a result that can be converted into another by means of `Into`.
pub trait ParserInto<I, O1: Into<O2>, O2, S>: Parser<I, O1, S> {
    /// Transform this parser into a parser with a different result. The existing type must
    /// implement `Into<R>` for the requested type `R`
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

    #[cfg(feature = "std")]
    fn debug(self, name: &'static str) -> impl Parser<I, O, S> {
        use std::println;

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

/// Perform a parse with provided user state.
///
/// ### Arguments
/// * `p` - the parser
/// * `input` - the input to be parsed
/// * `user_state` - the user state
pub fn parse_state<I, O, S>(p: impl Parser<I, O, S>,
                            input: I,
                            user_state: &mut S) -> AnpaResult<AnpaState<I, S>, O> {
    let mut parser_state = AnpaState { input, user_state };
    let result = p(&mut parser_state);
    AnpaResult { state: parser_state, result }
}

/// Perform a parse.
///
/// ### Arguments
/// * `p` - the parser
/// * `input` - the input to be parsed
pub fn parse<I, O>(p: impl Parser<I, O, ()>,
                   input: I) -> AnpaResult<I, O> {
    let mut parser_state = AnpaState { input, user_state: &mut () };
    let result = p(&mut parser_state);
    AnpaResult { state: parser_state.input, result }
}