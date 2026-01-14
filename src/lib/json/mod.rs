use std::string::String;
pub mod generator;
pub mod json_string;

use alloc::{collections::BTreeMap, vec::Vec};
pub use json_string::escaped_string_parser;

use crate::{combinators::*, core::parse_default, findbyte::{ByteFinder, eq, find_byte, lt}, number::float, parsers::*, whitespace::skip_ascii_whitespace};

create_parser_trait!(JsonParser, str, String, "Trait for a parser intended for JSON parsing, using a String user state to accumulate error messages.");

pub trait JsonDeserializable<'a, T> {
    fn json_parser() -> impl JsonParser<'a, T>;
    fn json_parser_exact() -> impl JsonParser<'a, T>;
}

#[inline(always)]
fn from_str_internal<'a, T>(p: impl JsonParser<'a, T>, input: &'a str) -> Result<T, String> {
    let res = parse_default(p, input);
    match res.result {
        Some(v) => Ok(v),
        None => Err(res.state.user_state)
    }
}

#[inline]
pub fn from_str<'a, T: JsonDeserializable<'a, T>>(input: &'a str) -> Result<T, String> {
    from_str_internal(T::json_parser(), input)
}

#[inline(always)]
pub fn from_str_exact<'a, T: JsonDeserializable<'a, T>>(input: &'a  str) -> Result<T, String> {
    from_str_internal(T::json_parser_exact(), input)
}

#[derive(Debug)]
pub enum JsonValue<StringType> {
    Null,
    Bool(bool),
    Str(StringType),
    Num(f64),
    Dic(BTreeMap<StringType, JsonValue<StringType>>),
    Arr(Vec<JsonValue<StringType>>)
}

pub const fn eat<'a, O>(p: impl JsonParser<'a, O>) -> impl JsonParser<'a, O> {
    right(skip_ascii_whitespace(), p)
}

pub(crate) fn string_element_finder() -> impl ByteFinder {
    eq(b'"') | eq(b'\\') | lt(0x20)
}

pub const fn string_parser<'a, T: From<&'a str>>() -> impl JsonParser<'a, T> {
    let unicode = right(skip!('u'), times(4, item_if(|c: char| c.is_ascii_hexdigit())));
    let escaped = right(item(), or_diff(item_matches!('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't'),
                                        unicode));
    let parse_until = choose!(find_byte(string_element_finder(), false);
                              b'\\' => escaped);
    into_type(middle(skip!('"'), many(parse_until, true, no_separator()), skip!('"')))
}

const fn json_string_parser<'a, T: From<&'a str>>() -> impl JsonParser<'a, JsonValue<T>> {
    map(string_parser(), JsonValue::Str)
}

const fn number_parser<'a, T>() -> impl JsonParser<'a, JsonValue<T>> {
    map(float(), JsonValue::Num)
}

const fn bool_parser<'a, T>() -> impl JsonParser<'a, JsonValue<T>> {
    or(map(skip!("true"), |_| JsonValue::Bool(true)), map(skip!("false"), |_| JsonValue::Bool(false)))
}

const fn null_parser<'a, T>() -> impl JsonParser<'a, JsonValue<T>> {
    map(skip!("null"), |_| JsonValue::Null)
}

/// Get a JSON parser that parses any JSON value. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
pub const fn value_parser<'a, T: From<&'a str> + Ord>() -> impl JsonParser<'a, JsonValue<T>> {
    eat(defer_parser! {
        or!(json_string_parser(), number_parser(), object_parser(),
            array_parser(), bool_parser(), null_parser())
    })
}

/// Get a JSON parser that parses a JSON object. The type used for strings will be inferred
/// from the context via `From<&str>`.
///
/// ### Example
/// ```
/// use anpa::json;
///
/// let p1 = json::object_parser::<&str>(); // Stores strings as slices of input
/// let p2 = json::object_parser::<String>(); // Stores strings as individual `String` instances.
///
/// // Stores strings as custom type implementing `From<&str>`.
/// // let p3 = json::object_parser::<MyString>();
/// ```
pub const fn object_parser<'a, T: From<&'a str> + Ord>() -> impl JsonParser<'a, JsonValue<T>> {
    let pair_parser = tuplify!(
        left(eat(string_parser()), colon_parser()),
        value_parser());
    map(middle(
        skip!('{'),
        many_to_map_ordered(pair_parser, true, separator(comma_parser(), false)),
        close_brace_parser()), JsonValue::Dic)
}

/// Get a JSON parser that parses a JSON array. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
#[inline]
pub const fn array_parser<'a, T: From<&'a str> + Ord>() -> impl JsonParser<'a, JsonValue<T>> {
    map(vec_parser(value_parser()), JsonValue::Arr)
}

/// Get a JSON parser that parses an array.
pub const fn vec_parser<'a, T>(p: impl JsonParser<'a, T>) -> impl JsonParser<'a, Vec<T>> {
    middle(
        skip!('['),
        many_to_vec(p, true, separator(comma_parser(), false)),
        eat(skip!(']')))
}

#[inline]
pub fn open_brace_parser<'a>() -> impl JsonParser<'a, ()> {
    eat(skip!('{'))
}

#[inline]
pub const fn close_brace_parser<'a>() -> impl JsonParser<'a, ()> {
    eat(skip!('}'))
}

#[inline]
pub const fn comma_parser<'a>() -> impl JsonParser<'a, ()> {
    eat(skip!(','))
}

#[inline]
pub const fn colon_parser<'a>() -> impl JsonParser<'a, ()> {
    eat(skip!(':'))
}

#[inline]
pub const fn option_parser<'a, T>(p: impl JsonParser<'a, T>) -> impl JsonParser<'a, Option<T>> {
    or(map(skip!("null"), |_| None), map(p, Some))
}

#[inline]
pub const fn bool_parse<'a>() -> impl JsonParser<'a, bool> {
    or(map(skip!("true"), |_| true), map(skip!("false"), |_| false))
}
