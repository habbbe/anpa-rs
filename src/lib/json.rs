use std::collections::HashMap;
use super::parsers::{*};
use super::combinators::{*};
use super::core::{*};

#[derive(Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Str(String),
    Num(f64),
    Dic(HashMap<String, JsonValue>),
    Arr(Vec<JsonValue>)
}

impl JsonValue {
    pub fn len(&self) -> usize {
        match self {
            JsonValue::Dic(d) => d.len(),
            JsonValue::Arr(a) => a.len(),
            _ => 0
        }
    }
}

fn eat<'a, O, S>(p: impl Parser<&'a str, O, S>) -> impl Parser<&'a str, O, S> {
    right(succeed(item_while(|c: char| c.is_whitespace())), p)
}

fn string_parser<'a>() -> impl Parser<&'a str, String, ()> {
    let unicode = right(item('u'), times(4, item_if(|c: char| c.is_digit(16))));
    let escaped = right(item('\\'), or_diff(unicode, item_if(|c: char| "\"\\/bfnrt".contains(c))));
    let not_end = or_diff(escaped, item_if(|c: char| c != '"' && !c.is_control()));
    middle(item('"'), many(not_end, true), item('"')).map(str::to_string)
}

fn json_string_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    string_parser().map(JsonValue::Str)
}

fn number_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    float_64().map(JsonValue::Num)
}

fn bool_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    or(seq("true").map(|_| JsonValue::Bool(true)), seq("false").map(|_| JsonValue::Bool(false)))
}

fn null_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    seq("null").map(|_| JsonValue::Null)
}

pub fn value_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    defer_parser! {
        eat(or!(json_string_parser(), number_parser(), object_parser(),
                array_parser(), bool_parser(), null_parser()))
    }
}

fn no_trailing_comma<'a, O, S>(p: impl Parser<&'a str, O, S>, terminator: char) -> impl Parser<&'a str, O, S> {
    left(p, eat(or(item(','), item(terminator))))
}

pub fn object_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    let pair_parser = tuplify!(
        left(eat(string_parser()), eat(item(':'))),
        value_parser());
    right(
        item('{'),
        many_to_map(no_trailing_comma(pair_parser, '}'), true).map(JsonValue::Dic))
}

pub fn array_parser<'a>() -> impl Parser<&'a str, JsonValue, ()> {
    right(
        item('['),
        many_to_vec(no_trailing_comma(value_parser(), ']'), true).map(JsonValue::Arr))
}