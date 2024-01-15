use std::num;

use crate::{parsers::{*}, core::{Parser, AnpaState, ParserExt}, combinators::{*}, number::integer};
#[derive(Debug)]
pub struct AnpaVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: String,
    pub build: String
}

impl AnpaVersion {
    pub fn new(major: u64, minor: u64, patch: u64, pre_release: impl Into<String>, build: impl Into<String>) -> AnpaVersion {
        AnpaVersion { major, minor, patch, pre_release: pre_release.into(), build: build.into() }
    }
}

pub fn semver<'a, S>() -> impl Parser<&'a str, AnpaVersion, S> {
    let number = integer();
    let component = left!(number, item('.'));
    let pre_release = or!(
        empty(), // If there is no pre-release or build
        peek(item('+')).map(|_| ""), // If there is a build (but no pre-release)
        right(item('-'), not_empty(item_while(|c| c != '+')))
    );
    let build = or(empty(), right(item('+'), not_empty(rest())));
    lift!(AnpaVersion::new,
        component,
        component,
        number,
        pre_release,
        build)
}

pub fn semver_strict<'a, S>() -> impl Parser<&'a str, AnpaVersion, S> {
    lift!(|(major, minor, patch), pre: Option<_>, build: Option<_>| {
        AnpaVersion::new(major, minor, patch, pre.unwrap_or(""), build.unwrap_or(""))
    }, version_core(), succeed(pre_release()), succeed(build())).left(empty())
}

fn version_core<'a, S>() -> impl Parser<&'a str, (u64, u64, u64), S> {
    let component = and_parsed(integer()).map_if(|(i, p): (&str, u64)| {
        (!i.starts_with('0') || p == 0).then_some(p)
    });

    // We could go completely with re-use, but it's faster to use the internal integer parser.
    // let num = numeric_identifier().map(|s| s.parse().unwrap());

    let major_minor = left(component, item('.'));
    tuplify!(major_minor, major_minor, component)
}

fn pre_release<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    dot_separated('-', pre_release_identifier())
}

fn build<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    dot_separated('+', build_identifier())
}

fn dot_separated<'a, S>(prefix: char, p: impl Parser<&'a str, &'a str, S>) -> impl Parser<&'a str, &'a str, S> {
    attempt(item(prefix).right(many(p, false, Some((false, item('.'))))))
}

fn pre_release_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    or(alphanumeric_identifier(), numeric_identifier())
}

fn build_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    not_empty(item_while(identifier_character))
}

fn alphanumeric_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    identifier_characters().filter(|s| !(s.len() == 1 && s.starts_with(digit)))
}

fn numeric_identifier<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    digits().filter(|d| d.len() == 1 || !d.starts_with('0'))
}

fn identifier_characters<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    not_empty(item_while(identifier_character))
}

fn identifier_character(c: char) -> bool {
    digit(c) || non_digit(c)
}

fn non_digit(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '-'
}

fn digits<'a, S>() -> impl Parser<&'a str, &'a str, S> {
    item_while(digit)
}

fn digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[cfg(test)]
mod tests {

    use crate::{semver::{semver_strict, semver}, core::parse};

    #[test]
    fn version_no_snapshot() {
        let res = parse(semver(), "1.2.3").1.unwrap();
        assert_eq!(res.major, 1);
        assert_eq!(res.minor, 2);
        assert_eq!(res.patch, 3);
        assert!(res.pre_release.is_empty());
    }

    #[test]
    fn version_snapshot() {
        let res = parse(semver(), "12.345.67890-SNAPSHOT").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT".to_string());

        assert!(parse(semver(), "12.345.67890-").1.is_none());
        assert!(parse(semver(), "12.345.67890-+").1.is_none());
        assert!(parse(semver(), "12.345.67890-SNAPSHOT+").1.is_none());
    }

    #[test]
    fn version_build() {
        let res = parse(semver(), "12.345.67890+build1").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert!(res.pre_release.is_empty());
        assert_eq!(res.build, "build1".to_string());
    }

    #[test]
    fn version_build_and_snapshot() {
        let res = parse(semver(), "12.345.67890-SNAPSHOT+build1").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT".to_string());
        assert_eq!(res.build, "build1".to_string());
    }
}