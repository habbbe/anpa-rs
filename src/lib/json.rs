use alloc::{collections::BTreeMap, vec::Vec};

use crate::{combinators::*, core::StrParser, findbyte::{eq, find_byte, lt}, number::float, parsers::*, whitespace::AsciiWhitespace};

#[derive(Debug)]
pub enum JsonValue<StringType> {
    Null,
    Bool(bool),
    Str(StringType),
    Num(f64),
    Dic(BTreeMap<StringType, JsonValue<StringType>>),
    Arr(Vec<JsonValue<StringType>>)
}

pub const fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
    // For unknown reasons, this gives better performance than `skip_ascii_whitespace()`.
    // Possibly a random optimization quirk, since it ideally shouldn't happen.
    right(skip!(AsciiWhitespace), p)
}

pub const fn string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, T> {
    let unicode = right(skip!('u'), times(4, item_if(|c: char| c.is_ascii_hexdigit())));
    let escaped = right(item(), or_diff(item_matches!('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't'),
                                        unicode));
    let parse_until = choose!(find_byte(eq(b'"') | eq(b'\\') | lt(0x20), false);
                                        b'\\' => escaped);
    into_type(middle(skip!('"'), many(parse_until, true, no_separator()), skip!('"')))
}

const fn json_string_parser<'a, T: From<&'a str>>() -> impl StrParser<'a, JsonValue<T>> {
    map(string_parser(), JsonValue::Str)
}

const fn number_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    map(float(), JsonValue::Num)
}

const fn bool_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    or(map(skip!("true"), |_| JsonValue::Bool(true)), map(skip!("false"), |_| JsonValue::Bool(false)))
}

const fn null_parser<'a, T>() -> impl StrParser<'a, JsonValue<T>> {
    map(skip!("null"), |_| JsonValue::Null)
}

/// Get a JSON parser that parses any JSON value. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
pub const fn value_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
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
pub const fn object_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    let pair_parser = tuplify!(
        left(eat(string_parser()), colon_parser()),
        value_parser());
    map(middle(
        skip!('{'),
        many_to_map_ordered(pair_parser, true, separator(eat(skip!(',')), false)),
        eat(skip!('}'))), JsonValue::Dic)
}

/// Get a JSON parser that parses a JSON array. The type used for strings will be inferred
/// from the context via `From<&str>`. For examples, see `object_parser`.
pub const fn array_parser<'a, T: From<&'a str> + Ord>() -> impl StrParser<'a, JsonValue<T>> {
    map(vec_parser(value_parser()), JsonValue::Arr)
}

/// Get a JSON parser that parses an array.
pub const fn vec_parser<'a, T>(p: impl StrParser<'a, T>) -> impl StrParser<'a, Vec<T>> {
    middle(
        skip!('['),
        many_to_vec(p, true, separator(comma_parser(), false)),
        eat(skip!(']')))
}

#[inline]
pub fn open_brace_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!('{'))
}

#[inline]
pub const fn close_brace_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!('}'))
}

#[inline]
pub const fn comma_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!(','))
}

#[inline]
pub const fn colon_parser<'a>() -> impl StrParser<'a, ()> {
    eat(skip!(':'))
}

#[inline]
pub const fn option_parser<'a, T>(p: impl StrParser<'a, T>) -> impl StrParser<'a, Option<T>> {
    or(map(skip!("null"), |_| None), map(p, Some))
}

#[inline]
pub const fn bool_parse<'a>() -> impl StrParser<'a, bool> {
    or(map(skip!("true"), |_| true), map(skip!("false"), |_| false))
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

/// Macro to generate a JSON parser for a specific structure. This parser
/// expects the exact structure given, i.e. no out-of-order fields, missing
/// fields, or additional fields. It will provide slightly better performance
/// than [`json_parser_gen_ng`].
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
/// use anpa::{core::parse, json_parser_gen, json, number};
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
///
/// let person = parse(person_parser, r#"{"name": "Anpa", "age": 28}"#).result.unwrap();
/// assert_eq!(person.name, "Anpa");
/// assert_eq!(person.age, 28);
/// ```
#[macro_export]
macro_rules! json_parser_gen {
    ($f:expr, ($id:expr, $parser:expr), $($rest:tt),*) => {
        $crate::combinators::middle(
            $crate::json::open_brace_parser(),
            $crate::map!($f,
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

#[macro_export]
macro_rules! const_if {
    (true, $e1:expr, $e2:expr) => {
        $e1
    };
    (false, $e1:expr, $e2:expr) => {
        $e2
    };
}

#[macro_export]
macro_rules! type_if_optional {
    (true, $t:ty) => {
        Option<$t>
    };
    (false, $t:ty) => {
        $t
    };
}

/// Macro to generate a JSON parser for a specific structure. Allows fields out of order.
///
/// ### Arguments
/// * `t` - The type to be parsed.
/// * `(name, id, id_ty, optional, parser)` - a variadic list of the elements to parse.
///   `name`: the field to parse
///   `id`: the field name in the result type
///   `id_ty`: The type of the field
///   `optional`: If the field is optional
///   `parser`: The parser for the field
///
/// ### Example
/// ```
/// use anpa::{core::parse, json_parser_gen_ng, json, number};
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// // The below will parse a `Person` object from a JSON string of the form:
/// // `{"name": "John Doe", "age": 27}`
/// let person_parser = json_parser_gen_ng!(Person,
///     ("name", name, String, false, json::string_parser()),
///     ("age", age, u8, false, number::integer())
/// );
///
/// let person = parse(person_parser, r#"{"age": 28, "name": "Anpa"}"#).result.unwrap();
/// assert_eq!(person.name, "Anpa");
/// assert_eq!(person.age, 28);
/// ```
#[macro_export]
macro_rules! json_parser_gen_ng {
    ($t:ident, $(($name:literal, $id:ident, $id_ty:ty, $optional:ident, $parser:expr)),* $(,)?) => {
        $crate::create_parser!(s, {
            #[allow(non_camel_case_types)]
            enum Variant {
                $($id($crate::type_if_optional!($optional, $id_ty)),)*
                Other
            }

            $(
                let $id = $crate::combinators::map(
                    $crate::right!(
                        $crate::skip!(concat!('\"', $name, '\"')),
                        $crate::json::eat($crate::skip!(':')),
                        $crate::json::eat($crate::const_if!($optional, $crate::json::option_parser($parser), $parser))), Variant::$id
                );
            )*

            // Parser used for fields not defined by the schema. Will be ignored
            let other = $crate::combinators::map(
                $crate::right!(
                    $crate::json::string_parser::<&str>(),
                    $crate::json::eat($crate::skip!(':')),
                    $crate::json::eat($crate::json::value_parser::<&str>())), |_| Variant::Other);

            $crate::json::eat($crate::skip!('{'))(s)?;

            let all_parser = $crate::or!($($id),*, other);

            $(
                let mut $id: Option<$crate::type_if_optional!($optional, $id_ty)> = $crate::const_if!($optional, Some(None), None);
            )*

            loop {
                $crate::whitespace::skip_ascii_whitespace()(s);

                let Some(res) = all_parser(s) else {
                    break;
                };

                match res {
                    $(Variant::$id(inner) => $id = Some(inner),)*
                    Variant::Other => {}
                }

                if $crate::json::eat($crate::skip!(','))(s).is_none() {
                    break;
                }
            }

            $crate::json::eat($crate::skip!('}'))(s)?;

            #[allow(unused)]
            fn unwrap<X>(o: Option<X>, n: &'static str) -> Option<X> {
                match o {
                    a@Some(_) => a,
                    None => {
                        // Here some message could be returned
                        None
                    }
                }
            }

            Some($t { $($id: unwrap($id, $name)?),* })
        })
    };
}
