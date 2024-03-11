use std::{collections::BTreeMap, vec::Vec};

use crate::{combinators::*, core::{ParserExt, ParserInto, StrParser}, findbyte::{eq, find_byte, lt}, number::float, parsers::*, whitespace::AsciiWhitespace};

#[derive(Debug)]
pub enum JsonValue<StringType> {
    Null,
    Bool(bool),
    Str(StringType),
    Num(f64),
    Dic(BTreeMap<StringType, JsonValue<StringType>>),
    Arr(Vec<JsonValue<StringType>>)
}

pub fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
    // For unknown reasons, this gives much better performance than `skip_ascii_whitespace()`.
    // Possibly a random optimization quirk, since it ideally shouldn't happen.
    right(skip!(AsciiWhitespace()), p)
}

pub fn string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, T> {
    let unicode = right(skip!('u'), times(4, item_if(|c: char| c.is_ascii_hexdigit())));
    let escaped = right(item(), or_diff(item_matches!('"', '\\', '/', 'b', 'f', 'n', 'r', 't'),
                                        unicode));
    let parse_until = choose!(find_byte(eq(b'"') | eq(b'\\') | lt(0x20), false);
                                        b'\\' => escaped);
    middle(skip!('"'), many(parse_until, true, no_separator()), skip!('"')).into_type()
}

pub fn json_string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, JsonValue<T>> {
    string_parser().map(JsonValue::Str)
}

pub fn number_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    float().map(JsonValue::Num)
}

pub fn bool_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    or(skip!("true").map(|_| JsonValue::Bool(true)), skip!("false").map(|_| JsonValue::Bool(false)))
}

pub fn null_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    skip!("null").map(|_| JsonValue::Null)
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
/// ```
/// use anpa::json;
///
/// let p1 = json::object_parser::<&str>(); // Stores strings as slices of input
/// let p2 = json::object_parser::<String>(); // Stores strings as individual `String` instances.
///
/// // Stores strings as custom type implementing `From<&str>`.
/// // let p3 = json::object_parser::<MyString>();
/// ```
pub fn object_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    let pair_parser = tuplify!(
        left(eat(string_parser()), colon_parser()),
        value_parser());
    middle(
        skip!('{'),
        many_to_map_ordered(pair_parser, true, separator(comma_parser(), false)),
        close_brace_parser()).map(JsonValue::Dic)
}

/// Get a JSON parser that parses a JSON array. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
pub fn array_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    middle(
        skip!('['),
        many_to_vec(value_parser(), true, separator(comma_parser(), false)),
        eat(skip!(']'))).map(JsonValue::Arr)
}

pub fn open_brace_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!('{'))
}

pub fn close_brace_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!('}'))
}

pub fn comma_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!(','))
}

pub fn colon_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!(':'))
}

#[macro_export]
macro_rules! internal_json_field {
    (($id:expr, $parser:expr)) => {
        $crate::right!(
            $crate::json::eat($crate::skip!(concat!('"', $id, '"'))),
            $crate::json::colon_parser(),
            $crate::json::eat($parser)
        )
    };
}

/// Macro to generate a JSON parser for a specific structure.
///
/// ### Arguments
/// * `f` - A function returning the structure from the arguments parsed by the
///         subsequent arguments.
/// * `(id, parser)` - a variadic list of the elements to parse. `id` is the name of
///                    the string-value pair, and `parser` is a parser that can parse
///                    the expected value.
///
/// ### Example
/// ```
/// use anpa::{json_parser_gen, json, number};
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// // The below will parse a `Person` object from a JSON string of the form:
/// // `{"name": "John Doe", "age": 27}`
/// let person_parser = json_parser_gen!(|name, age| Person { name, age },
///     ("name", json::string_parser()),
///     ("age", number::integer())
/// );
/// ```
#[macro_export]
macro_rules! json_parser_gen {
    ($f:expr, ($id:expr, $parser:expr), $($rest:tt),*) => {
        $crate::combinators::middle(
            $crate::json::open_brace_parser(),
            $crate::lift!($f,
                $crate::internal_json_field!(($id, $parser)),
                $($crate::combinators::right(
                    $crate::json::comma_parser(),
                    $crate::internal_json_field!($rest)
                )),*
            ),
            $crate::json::close_brace_parser()
        )
    };
}