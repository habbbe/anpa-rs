use crate::{parsers::{*}, core::{Parser, AnpaState, ParserExt, parse}, combinators::{*}, number::integer};
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

pub fn parse_version_general<'a, O: From<&'a str>>(text: &'a str) -> Option<AnpaVersion<O>> {
    parse(semver(), text.into()).result
}

pub fn parse_version_inline(text: &str) -> Option<AnpaVersion<&str>> {
    parse_version_general(text)
}

pub fn parse_version(text: &str) -> Option<AnpaVersion<String>> {
    parse_version_general(text)
}

#[inline]
pub fn semver<'a, T: From<&'a str>>() -> impl Parser<&'a str, AnpaVersion<T>, ()> {
    lift!(|(major, minor, patch), pre: Option<_>, build: Option<_>| {
        AnpaVersion::new(major, minor, patch, pre.unwrap_or(""), build.unwrap_or(""))
    }, version_core(), succeed(pre_release()), succeed(build())).left(empty())
}

#[inline]
fn version_core<'a, S>() -> impl Parser<&'a str, (u64, u64, u64), S> {
    let component = and_parsed(integer()).map_if(|(i, p): (&str, _)| {
        (!i.starts_with('0') || p == 0).then_some(p)
    });

    // We could go completely with re-use, but it's faster to use the internal integer parser.
    // let component = numeric_identifier().map(|s| s.parse().unwrap());

    let major_minor = left(component, item('.'));
    tuplify!(major_minor, major_minor, component)
}

#[inline]
fn pre_release<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    dot_separated('-', pre_release_identifier())
}

#[inline]
fn build<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    dot_separated('+', build_identifier())
}

#[inline]
fn dot_separated<'a, S>(prefix: char, p: impl Parser<&'a str, &'a str, S>) -> impl Parser<&'a str, &'a str, S> {
    attempt(item(prefix).right(many(p, false, separator(item('.'), false))))
}

#[inline]
fn pre_release_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    or(alphanumeric_identifier(), numeric_identifier())
}

#[inline]
fn build_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    identifier_characters()
}

#[inline]
fn alphanumeric_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    get_parsed(succeed(digits()).right(identifier_characters()))
}

#[inline]
fn numeric_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    not_empty(digits()).filter(|d| d.len() == 1 || !d.starts_with('0'))
}

#[inline]
fn identifier_characters<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    not_empty(item_while(identifier_character))
}

#[inline]
fn identifier_character(c: char) -> bool {
    digit(c) || non_digit(c)
}

#[inline]
fn non_digit(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '-'
}

#[inline]
fn digits<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    item_while(digit)
}

#[inline]
fn digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[cfg(test)]
mod tests {

    use crate::semver::parse_version;

    #[test]
    fn version_no_snapshot() {
        let res = parse_version("1.2.3").unwrap();
        assert_eq!(res.major, 1);
        assert_eq!(res.minor, 2);
        assert_eq!(res.patch, 3);
        assert!(res.pre_release.is_empty());
    }

    #[test]
    fn version_snapshot() {
        let res = parse_version("12.345.67890-SNAPSHOT").unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT".to_string());

        assert!(parse_version("12.345.67890-").is_none());
        assert!(parse_version("12.345.67890-+").is_none());
        assert!(parse_version("12.345.67890-+build").is_none());
        assert!(parse_version("12.345.67890-SNAPSHOT+").is_none());
    }

    #[test]
    fn version_build() {
        let res = parse_version("12.345.67890+build1").unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert!(res.pre_release.is_empty());
        assert_eq!(res.build, "build1".to_string());
    }

    #[test]
    fn version_build_and_snapshot() {
        let res = parse_version("12.345.67890-SNAPSHOT+build1").unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT".to_string());
        assert_eq!(res.build, "build1".to_string());
    }
}