use std::collections::BTreeMap;

use crate::{combinators::*, core::{ParserExt, ParserInto, StrParser}, number::float, parsers::*};

#[derive(Debug)]
pub enum JsonValue<StringType> {
    Null,
    Bool(bool),
    Str(StringType),
    Num(f64),
    Dic(BTreeMap<StringType, JsonValue<StringType>>),
    Arr(Vec<JsonValue<StringType>>)
}

fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
    right(succeed(item_while(|c: char| c.is_whitespace())), p)
}

fn string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, T> {
    let unicode = right(item!('u'), times(4, item_if(|c: char| c.is_digit(16))));
    let escaped = right(item!('\\'), or_diff(unicode, item_if(|c: char| "\"\\/bfnrt".contains(c))));
    let valid_char = item_if(|c: char| c != '"' && c != '\\' && !c.is_control());
    let not_end = or_diff(valid_char, escaped);
    middle(item!('"'), many(not_end, true, no_separator()), item!('"')).into_type()
}

fn json_string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, JsonValue<T>> {
    string_parser().map(JsonValue::Str)
}

fn number_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    float().map(JsonValue::Num)
}

fn bool_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    or(seq("true").map(|_| JsonValue::Bool(true)), seq("false").map(|_| JsonValue::Bool(false)))
}

fn null_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    seq("null").map(|_| JsonValue::Null)
}

/// Get a JSON parser that parses any JSON value. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
pub fn value_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    defer_parser! {
        eat(or!(json_string_parser(), number_parser(), object_parser(),
                array_parser(), bool_parser(), null_parser()))
    }
}

/// Get a JSON parser that parses a JSON object. The type used for strings will be inferred
/// from the context via `From<&str>`.
///
/// ### Example
/// ```ignore
/// let p1 = json::object_parser::<&str>(); // Stores strings as slices of input
/// let p2 = json::object_parser::<String>(); // Stores strings as individual `String` instances.
/// let p2 = json::object_parser::<MyString>(); // Stores strings as custom type implementing `From<&str>`.
/// ```
pub fn object_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    let pair_parser = tuplify!(
        left(eat(string_parser()), eat(item!(':'))),
        value_parser());
    middle(
        item!('{'),
        many_to_map_ordered(pair_parser, true, separator(eat(item!(',')), false)),
        eat(item!('}'))).map(JsonValue::Dic)
}

/// Get a JSON parser that parses a JSON array. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
pub fn array_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    middle(
        item!('['),
        many_to_vec(value_parser(), true, separator(eat(item!(',')), false)),
        eat(item!(']'))).map(JsonValue::Arr)
}