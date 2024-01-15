use std::collections::BTreeMap;

use crate::number::float;
use super::parsers::{*};
use super::combinators::{*};
use super::core::{*};

#[derive(Debug)]
pub enum JsonValue<StringType> {
    Null,
    Bool(bool),
    Str(StringType),
    Num(f64),
    Dic(BTreeMap<StringType, JsonValue<StringType>>),
    Arr(Vec<JsonValue<StringType>>)
}

fn eat<'a, O, S>(p: impl Parser<&'a str, O, S>) -> impl Parser<&'a str, O, S> {
    right(succeed(item_while(|c: char| c.is_whitespace())), p)
}

fn string_parser<'a, T: From<&'a str>>() -> impl Parser<&'a str, T, ()> {
    let unicode = right(item('u'), times(4, item_if(|c: char| c.is_digit(16))));
    let escaped = right(item('\\'), or_diff(unicode, item_if(|c: char| "\"\\/bfnrt".contains(c))));
    let valid_char = item_if(|c: char| c != '"' && c != '\\' && !c.is_control());
    let not_end = or_diff(valid_char, escaped);
    middle(item('"'), many(not_end, true, no_separator()), item('"')).into_type()
}

fn json_string_parser<'a, T: From<&'a str>>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    string_parser().map(JsonValue::Str)
}

fn number_parser<'a, T>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    float().map(JsonValue::Num)
}

fn bool_parser<'a, T>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    or(seq("true").map(|_| JsonValue::Bool(true)), seq("false").map(|_| JsonValue::Bool(false)))
}

fn null_parser<'a, T>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    seq("null").map(|_| JsonValue::Null)
}

pub fn value_parser<'a, T: From<&'a str> + Ord>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    defer_parser! {
        eat(or!(json_string_parser(), number_parser(), object_parser(),
                array_parser(), bool_parser(), null_parser()))
    }
}

pub fn object_parser<'a, T: From<&'a str> + Ord>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    let pair_parser = tuplify!(
        left(eat(string_parser()), eat(item(':'))),
        value_parser());
    middle(
        item('{'),
        many_to_map_ordered(pair_parser, true, Some((false, eat(item(','))))),
        eat(item('}'))).map(JsonValue::Dic)
}

pub fn array_parser<'a, T: From<&'a str> + Ord>() -> impl Parser<&'a str, JsonValue<T>, ()> {
    middle(
        item('['),
        many_to_vec(value_parser(), true, Some((false, eat(item(','))))),
        eat(item(']'))).map(JsonValue::Arr)
}