
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
/// fields, or additional fields. It will provide slightly better performance than
/// [`json_parser_gen`].
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
/// use anpa::{core::parse_default, json_parser_gen_exact, json, number};
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// // The below will parse a `Person` object from a JSON string of the form:
/// // `{"name": "John Doe", "age": 27}`
/// let person_parser = json_parser_gen_exact!(|name, age| Person { name, age },
///     ("name", json::string_parser()),
///     ("age", number::integer())
/// );
///
/// let person = parse_default(person_parser, r#"{"name": "Anpa", "age": 28}"#).result.unwrap();
/// assert_eq!(person.name, "Anpa");
/// assert_eq!(person.age, 28);
/// ```
#[macro_export]
macro_rules! json_parser_gen_exact {
    ($f:expr, ($id:expr, $parser:expr) $(, $rest:tt)* $(,)?) => {
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
        $crate::const_if!($e1, $e2)
    };
    ($e1:expr, $e2:expr) => {
        $e2
    };
}

#[macro_export]
macro_rules! type_if_optional {
    (true, $t:ty) => {
        Option<$t>
    };
    (false, $t:ty) => {
        $crate::type_if_optional!($t)
    };
    ($t:ty) => {
        $t
    };
}

#[macro_export]
macro_rules! if_more_than_10_fields {
    ($a:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt, $e:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt, $e:tt, $f:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt, $e:tt, $f:tt, $g:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt, $e:tt, $f:tt, $g:tt, $h:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt, $e:tt, $f:tt, $g:tt, $h:tt, $i:tt; $e1:expr, $e2:expr) => { $e2 };
    ($a:tt, $b:tt, $c:tt, $d:tt, $e:tt, $f:tt, $g:tt, $h:tt, $i:tt, $j:tt; $e1:expr, $e2:expr) => { $e2 };
    ($($rest:tt),*; $e1:expr, $e2:expr) => { $e1 };
}

/// Macro to generate a JSON parser for a specific structure. Allows fields out of order.
///
/// The `optional` field can be omitted, and the field will then be considered mandatory.
///
/// ### Arguments
/// * `t` - The type to be parsed.
/// * `(name, id, id_ty, parser, optional: opt)` - a variadic list of the elements to parse.
///   `name`: the field to parse
///   `id`: the field name in the result type
///   `id_ty`: The type of the field
///   `parser`: The parser for the field
///   `opt`: If the field is optional
///
/// ### Example
/// ```
/// use anpa::{core::parse_default, json_parser_gen, json, number};
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// // The below will parse a `Person` object from a JSON string of the form:
/// // `{"name": "John Doe", "age": 27}`
/// let person_parser = json_parser_gen!(Person,
///     ("name", name, String, json::string_parser(), optional: false),
///     ("age", age, u8, number::integer())
/// );
///
/// let person = parse_default(person_parser, r#"{"age": 28, "name": "Anpa"}"#).result.unwrap();
/// assert_eq!(person.name, "Anpa");
/// assert_eq!(person.age, 28);
/// ```
#[macro_export]
macro_rules! json_parser_gen {
    ($($lt:lifetime,)? $t:ident, $(($name:literal, $id:ident, $id_ty:ty, $parser:expr $(, optional: $optional:tt)?)),* $(,)?) => {
        $crate::create_parser!(s, {
            // Internal enum that contains a variant for each field, plus a wildcard that arbitrary
            // values can be parsed into.

            // Simplify implementation by allowing the field ID to be reused as the variant name,
            // even though it's likely not CamelCase.
            #[allow(non_camel_case_types)]
            enum Variant$(<$lt>)? {
                $($id($crate::type_if_optional!($($optional,)? $id_ty)),)*
                Other
            }

            let str_parser = $crate::json::string_parser::<&str>();
            let any_parser = $crate::json::eat($crate::json::value_parser::<&str>());
            let colon_parser = $crate::json::colon_parser();

            // Create the parser for each field on the format `"field_name": type_parser`.
            // For optional fields, the parser will be wrapped in an `option_parser`.
            // Each parser will return successful results in its respective `Variant`.
            //
            // Separate parsing strategies will be employed depending on the number of fields.
            let all_parser = $crate::if_more_than_10_fields!($($name),*;
                $crate::choose!(
                    $crate::left!(str_parser, colon_parser);
                    $($name => $crate::json::eat(
                        $crate::combinators::map(
                            $crate::const_if!($($optional,)? $crate::json::option_parser($parser), $parser),
                            Variant::$id
                        )
                    ),)*
                    _ => $crate::combinators::map(any_parser, |_| Variant::Other)
                ),
                ({
                    $(
                        let $id =
                            $crate::right!(
                                $crate::skip!(concat!('\"', $name, '\"')),
                                colon_parser,
                                $crate::json::eat(
                                    $crate::combinators::map(
                                        $crate::const_if!($($optional,)? $crate::json::option_parser($parser), $parser),
                                        Variant::$id
                                    )
                                )
                            );
                        ;
                    )*

                    // Parser used for fields not defined by the schema. Will be ignored
                    let other = $crate::combinators::map($crate::right!(str_parser, colon_parser, any_parser),
                                                         |_| Variant::Other);

                    $crate::or!($($id),*, other)
                })
            );

            $crate::json::open_brace_parser()(s)?;

            // Create a `Option<Type>` variable for all fields.
            // If optional the type will instead be `Option<Option<Type>>`, and
            // will be initialized as the successful status `Some(None)`.
            $(
                let mut $id: Option<$crate::type_if_optional!($($optional,)? $id_ty)> =
                    $crate::const_if!($($optional,)? Some(None), None);
            )*

            let whitespace_parser = $crate::whitespace::skip_ascii_whitespace();
            let comma_parser = $crate::json::comma_parser();

            loop {
                whitespace_parser(s);

                let Some(res) = all_parser(s) else {
                    break;
                };

                match res {
                    $(Variant::$id(inner) => $id = Some(inner),)*
                    _ => {}
                }

                if comma_parser(s).is_none() {
                    break;
                }
            }

            $crate::json::close_brace_parser()(s)?;

            match ($($id),*) {
                ($(Some($id)),*) => Some($t { $($id: $id),* }),
                ($($id),*) => {
                    $(
                        if $id.is_none() {
                            s.user_state.push_str(
                                concat!("Field \"", $name, "\" missing or invalid in ", stringify!($t) ,"\n"));
                        }
                    )*
                    None
                }
            }
        })
    };
}
