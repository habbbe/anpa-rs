/// Shorthand for creating a parser.
/// ### Example:
/// ```ignore
/// let p = create_parser!(s, { /* Define parser behavior using state `s` */ })
/// ```
#[macro_export]
macro_rules! create_parser {
    ($state:ident, $f:expr) => {
        move |$state: &mut $crate::core::AnpaState<_, _>| $f
    }
}

/// Shorthand for creating a deferred parser. This is mandatory when creating recursive parsers.
///
/// ### Example:
/// ```
/// use anpa::defer_parser;
/// use anpa::core::*;
/// use anpa::combinators::{middle, not_empty, or};
/// use anpa::parsers::{item_while, take};
///
/// /// Can parse e.g. "(((something)))"
/// fn in_parens<'a>() -> impl StrParser<'a> {
///     defer_parser!(or(
///         not_empty(item_while(|c: char| c.is_alphanumeric())),
///         middle(take('('), in_parens(), take(')')))
///     )
/// }
///
/// assert_eq!(parse(in_parens(), "((((inside))))" ).result, Some("inside"));
///
/// ```
#[macro_export]
macro_rules! defer_parser {
    ($p:expr) => {
        $crate::create_parser!(s, $p(s))
    }
}

/// Variadic version of `map`, where all provided parsers must succeed.
/// ### Arguments
/// * `f` - the transformation function. Its arguments must match the result types of `p...` in
///         both type and number.
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! map {
    ($f:expr, $($p:expr),* $(,)?) => {
        $crate::create_parser!(s, Some($f($($p(s)?),*)))
    };
}

/// Variadic version of `map_if`, where all provided parsers must succeed.
/// ### Arguments
/// * `f` - the transformation function. Its arguments must match the result types of `p...` in
///         both type and number.
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! map_if {
    ($f:expr, $($p:expr),* $(,)?) => {
        $crate::create_parser!(s, Some($f($($p(s)?),*)?))
    };
}

/// Convert a number of parsers to a single parser producing a tuple with all the results.
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! tuplify {
    ($($p:expr),* $(,)?) => {
        $crate::create_parser!(s, Some(($($p(s)?),*)))
    };
}

/// Create a parser that successfully returns `x`.
///
/// ### Arguments
/// * `x` - the result to be returned from the parser.
#[macro_export]
macro_rules! pure {
    ($x:expr) => {
        $crate::create_parser!(_s, Some($x))
    };
}

/// A helper macro to generate variadic macros using repeated application of the rightmost
/// argument of a binary function.
///
/// E.g. for 4 arguments, the resulting function will be constructed as:
/// `f(e1, f(e2, f(e3, e4)))`
///
/// ### Arguments
/// * `f` - a binary function.
/// * `e...` - any number of arguments.
#[macro_export]
macro_rules! variadic {
    ($f:expr, $e:expr) => {
        $e
    };
    ($f:expr, $e:expr, $($e2:expr),*) => {
        $f($e, $crate::variadic!($f, $($e2),*))
    };
}

/// Variadic version of `or`.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! or {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::or, $($p),*)
    };
}

/// Variadic version of `or_no_partial`.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! or_no_partial {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::or_no_partial, $($p),*)
    };
}

/// Variadic version of `or_diff`.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! or_diff {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::or_diff, $($p),*)
    };
}

/// Variadic version of `or_diff_no_partial`.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! or_diff_no_partial {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::or_diff_no_partial, $($p),*)
    };
}

/// Variadic version of `left`, where only the leftmost parser's result will be returned.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! left {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::left, $($p),*)
    };
}

/// Variadic version of `right`, where only the rightmost parser's result will be returned.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! right {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::right, $($p),*)
    };
}

/// Variadic parser that succeeds if the next item matches one of the
/// provided arguments.
///
/// ### Arguments
/// * `pattern` - any number of match patterns.
///
/// ### Example:
/// ```
/// use anpa::core::*;
/// use anpa::item_matches;
///
/// let p = item_matches!('0', '1');
///
/// assert_eq!(parse(p, "012").result, Some('0'));
/// assert_eq!(parse(p, "123").result, Some('1'));
/// assert_eq!(parse(p, "234").result, None);
/// ```
#[macro_export]
macro_rules! item_matches {
    ($($pattern:pat_param $(if $guard:expr)?),+ $(,)?) => {
        $crate::parsers::item_if(|c| matches!(c, $($pattern $(if $guard)?) |*))
    };
}

/// Alternative to the `take` parser that inlines the argument into the parser.
///
/// This can give better performance and/or smaller binary size, or the opposite.
/// Try it and don't forget to measure!
///
/// This macro is likely only useful when passing a literal as argument.
///
/// ### Arguments
/// * `prefix` - the prefix to parse.
#[macro_export]
macro_rules! take {
    ($prefix:expr) => {
        $crate::create_parser!(s, {
            $crate::prefix::Prefix::take_prefix(&$prefix, s.input).map(|(res, rest)| {
                s.input = rest;
                res
            })
        })
    }
}

/// Alternative to the `skip` parser that inlines the argument into the parser.
///
/// This can give better performance and/or smaller binary size, or the opposite.
/// Try it and don't forget to measure!
///
/// This macro is likely only useful when passing a literal as argument.
///
/// ### Arguments
/// * `prefix` - the prefix to parse.
#[macro_export]
macro_rules! skip {
    ($prefix:expr) => {
        $crate::create_parser!(s, {
            s.input = $crate::prefix::Prefix::skip_prefix(&$prefix, s.input)?;
            Some(())
        })
    }
}
/// Alternative to the `until` parser that inlines the argument into the parser.
///
/// This can give better performance and/or smaller binary size, or the opposite.
/// Try it and don't forget to measure!
///
/// This macro is likely only useful when passing a literal as argument.
///
/// ### Arguments
/// * `needle` - the element to search for.
#[macro_export]
macro_rules! until {
    ($needle:expr) => {
        $crate::create_parser!(s, {
            let (size, index) = $crate::needle::Needle::find_in(&$needle, s.input)?;
            let res = $crate::slicelike::SliceLike::slice_to(s.input, index);
            s.input = $crate::slicelike::SliceLike::slice_from(s.input, index + size);
            Some(res)
        })
    }
}

/// Variadic version of `greedy_or`, where the result of the parser with the most consumed
/// input will be returned.
///
/// ### Arguments
/// * `p...` - any number of parsers.
#[macro_export]
macro_rules! greedy_or {
    ($($p:expr),* $(,)?) => {
        $crate::variadic!($crate::combinators::greedy_or, $($p),*)
    };
}

/// Create a parser that takes the result of a parser, and returns different
/// parsers depending on the provided conditions.
///
/// If none of the provided conditions match, the parser will fail.
///
/// The macro comes in two flavors:
/// - if-style:
/// ```
/// # use anpa::{choose, core::parse, number::integer, parsers::take};
///     let p = choose!(integer() => n; // Explicit binding to variable `v`
///                    (0_u8..=4).contains(&n) => take("range"),
///                     n == 5                 => take("five"));
/// # parse(p, "dummy");
/// ```
/// - match-style:
/// ```
/// # use anpa::{choose, core::parse, number::integer, parsers::take};
///     let p = choose!(integer(); // No binding of result.
///                     0_u8..=4 => take("range"),
///                     5        => take("five"));
/// # parse(p, "dummy");
/// ```
///
///
/// ### Example:
/// ```
/// use anpa::core::*;
/// use anpa::choose;
/// use anpa::parsers::take;
/// use anpa::number::integer;
///
/// let p_if = choose!(integer() => x: u8; // Note the semicolon
///                   x == 0 => take("zero"),
///                   x == 1 => take("one"),
///                   (2..=100).contains(&x) => take("big")
/// );
///
/// let p_match = choose!(integer(); // Note the semicolon
///                       0_u8 => take("zero"),
///                       1 => take("one"),
///                       2..=100 => take("big")
/// );
///
/// let input1 = "0zero";
/// let input2 = "1one";
/// let input3 = "2big";
/// let input4 = "100big";
/// let input5 = "101big";
/// let input6 = "0one";
/// let input7 = "1zero";
///
/// assert_eq!(parse(p_if, input1).result, Some("zero"));
/// assert_eq!(parse(p_if, input2).result, Some("one"));
/// assert_eq!(parse(p_if, input3).result, Some("big"));
/// assert_eq!(parse(p_if, input4).result, Some("big"));
/// assert_eq!(parse(p_if, input5).result, None);
/// assert_eq!(parse(p_if, input6).result, None);
/// assert_eq!(parse(p_if, input7).result, None);
///
/// assert_eq!(parse(p_match, input1).result, Some("zero"));
/// assert_eq!(parse(p_match, input2).result, Some("one"));
/// assert_eq!(parse(p_match, input3).result, Some("big"));
/// assert_eq!(parse(p_match, input4).result, Some("big"));
/// assert_eq!(parse(p_match, input5).result, None);
/// assert_eq!(parse(p_match, input6).result, None);
/// assert_eq!(parse(p_match, input7).result, None);
/// ```
#[macro_export]
macro_rules! choose {
    ($p:expr => $res:ident $(: $t:ty)?; $($cond:expr => $new_p:expr),* $(,)?) => {
        $crate::create_parser!(s, {
            let $res $(:$t)? = $p(s)?;

            $(if $cond {
                return $new_p(s)
            })*

            None
        })
    };

    ($p:expr; $($arm:pat_param $(| $alt:pat_param)* $(if $guard:expr)? => $new_p:expr),* $(,)?) => {
        $crate::create_parser!(s, {
            match $p(s)? {
                $(
                    $arm $(| $alt)* $(if $guard)? => $new_p(s),
                )*
                #[allow(unreachable_patterns)]
                _ => None
            }
        })
    };
}

/// Create a new parser trait with a concrete input type for cleaner APIs.
/// ### Arguments
/// * `id` - The identifier of the new trait
/// * `input` - The type of the input. Do not include lifetime or reference.
/// * `comment` - The doc comment to be generated for the trait
///
/// ### Example
/// ```
/// use anpa::create_parser_trait;
/// create_parser_trait!(I8Parser, [i8], "Convenience alias for a parser that parses a `&'a [i8]`");
/// ```
#[macro_export]
macro_rules! create_parser_trait {
    ($id:ident, $input:ty, $comment:expr) => {
        #[doc=$comment]
        pub trait $id<'a, O = &'a $input, S = ()>: $crate::core::Parser<&'a $input, O, S> {}
        impl<'a, O, S, P: $crate::core::Parser<&'a $input, O, S>> $id<'a, O, S> for P {}
    };
}