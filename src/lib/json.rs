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

pub fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
    right(succeed(item_while(|c: char| c.is_whitespace())), p)
}

pub fn string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, T> {
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

pub fn open_brace_parser<'a>() -> impl StrParser<'a, char> {
    eat(item!('{'))
}

pub fn close_brace_parser<'a>() -> impl StrParser<'a, char> {
    eat(item!('}'))
}

pub fn comma_parser<'a>() -> impl StrParser<'a, char> {
    eat(item!(','))
}

pub fn colon_parser<'a>() -> impl StrParser<'a, char> {
    eat(item!(':'))
}

#[macro_export]
macro_rules! internal_json_field {
    (($id:expr, $parser:expr)) => {
        $crate::right!(
            $crate::json::eat($crate::parsers::seq(concat!("\"", $id, "\""))),
            $crate::json::colon_parser(),
            $crate::json::eat($parser)
        )
    };
}

/// Macro to generate a JSON parser for a specific structure.
///
/// ### Arguments
/// * `f` - A function with the same number of arguments as the number of elements in
///         the structure.
/// * `(id, parser)` - a variadic list of the elements to parse. `id` is the name of
///                    the string-value pair, and `parser` is how to parse the value.
///
/// ### Example
/// ```ignore
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// // The below will parse a `Person` object from a JSON string of the form:
/// // `{"name": "John Doe", "age": 27}`
/// let person_parser = json_parser_gen!(|name, age| Person { name, age },
///     ("name", json_string_parser()),
///     ("age": number())
/// );
/// ```
#[macro_export]
macro_rules! json_parser_gen {
    ($f:expr, ($id:expr, $parser:expr), $($rest:tt),*) => {
        $crate::combinators::middle(
            $crate::json::open_brace_parser(),
            lift!($f,
                internal_json_field!(($id, $parser)),
                $($crate::combinators::right(
                    $crate::json::comma_parser(),
                    internal_json_field!($rest)
                )),*
            ),
            $crate::json::close_brace_parser()
        )
    };
}