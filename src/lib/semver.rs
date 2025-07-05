use crate::{combinators::*, core::{ParserExtNoState, StrParser}, number::integer, parsers::*};

#[derive(Debug)]
pub struct AnpaVersion<T> {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: T,
    pub build: T
}

impl<T> AnpaVersion<T> {
    pub fn new(major: u64, minor: u64, patch: u64, pre_release: impl Into<T>, build: impl Into<T>) -> AnpaVersion<T> {
        AnpaVersion { major, minor, patch, pre_release: pre_release.into(), build: build.into() }
    }
}

/// Parse a SemVer string from `text`. General version that infer the `pre_release` and `build` type
/// by means of `From<&str>`.
pub fn parse_general<'a, O: From<&'a str>>(text: &'a str) -> Option<AnpaVersion<O>> {
    semver().parse(text).result
}

/// Parse a SemVer string from `text`. `pre_release` and `build` will be stored as slices
/// of the input.
pub fn parse_inline(text: &str) -> Option<AnpaVersion<&str>> {
    parse_general(text)
}

#[cfg(feature = "std")]
/// Parse a SemVer string from `text`. `pre_release` and `build` will be stored as independent
/// `String` values.
pub fn parse(text: &str) -> Option<AnpaVersion<std::string::String>> {
    parse_general(text)
}

#[inline]
pub const fn semver<'a, T: From<&'a str>>() -> impl StrParser<'a, AnpaVersion<T>> {
    left(map!(|(major, minor, patch), pre: Option<_>, build: Option<_>| {
        AnpaVersion::new(major, minor, patch, pre.unwrap_or(""), build.unwrap_or(""))
    }, version_core(), succeed(pre_release()), succeed(build())), empty())
}

#[inline]
const fn version_core<'a>() -> impl StrParser<'a, (u64, u64, u64)> {
    let component = map_if(and_parsed(integer()), |(i, p): (&str, _)| {
        (!i.starts_with('0') || p == 0).then_some(p)
    });

    // We could go completely with re-use, but it's faster to use the internal integer parser.
    // let component = numeric_identifier().map(|s| s.parse().unwrap());

    let major_minor = left(component, skip('.'));
    tuplify!(major_minor, major_minor, component)
}

#[inline]
const fn pre_release<'a>() -> impl StrParser<'a> {
    dot_separated('-', pre_release_identifier())
}

#[inline]
const fn build<'a>() -> impl StrParser<'a> {
    dot_separated('+', build_identifier())
}

#[inline]
const fn dot_separated<'a>(prefix: char, p: impl StrParser<'a>) -> impl StrParser<'a> {
    attempt(right(skip(prefix), many(p, false, separator(skip('.'), false))))
}

#[inline]
const fn pre_release_identifier<'a>() -> impl StrParser<'a> {
    or(alphanumeric_identifier(), numeric_identifier())
}

#[inline]
const fn build_identifier<'a>() -> impl StrParser<'a> {
    identifier_characters()
}

#[inline]
const fn alphanumeric_identifier<'a>() -> impl StrParser<'a> {
    get_parsed(right(digits(), identifier_characters()))
}

#[inline]
const fn numeric_identifier<'a>() -> impl StrParser<'a> {
    filter(not_empty(digits()), |d| d.len() == 1 || !d.starts_with('0'))
}

#[inline]
const fn identifier_characters<'a>() -> impl StrParser<'a> {
    not_empty(item_while(identifier_character))
}

#[inline]
const fn identifier_character(c: char) -> bool {
    digit(c) || non_digit(c)
}

#[inline]
const fn non_digit(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '-'
}

#[inline]
const fn digits<'a>() -> impl StrParser<'a> {
    item_while(digit)
}

#[inline]
const fn digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use crate::semver::parse_inline;

    #[test]
    fn version_no_snapshot() {
        let res = parse_inline("1.2.3").unwrap();
        assert_eq!(res.major, 1);
        assert_eq!(res.minor, 2);
        assert_eq!(res.patch, 3);
        assert!(res.pre_release.is_empty());
    }

    #[test]
    fn version_snapshot() {
        let res = parse_inline("12.345.67890-SNAPSHOT").unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT");

        assert!(parse_inline("12.345.67890-").is_none());
        assert!(parse_inline("12.345.67890-+").is_none());
        assert!(parse_inline("12.345.67890-+build").is_none());
        assert!(parse_inline("12.345.67890-SNAPSHOT+").is_none());
    }

    #[test]
    fn version_build() {
        let res = parse_inline("12.345.67890+build1").unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert!(res.pre_release.is_empty());
        assert_eq!(res.build, "build1");
    }

    #[test]
    fn version_build_and_snapshot() {
        let res = parse_inline("12.345.67890-SNAPSHOT+build1").unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT");
        assert_eq!(res.build, "build1");
    }
}