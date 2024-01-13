use crate::{parsers::{*}, core::{Parser, AnpaState, ParserExt}, combinators::{*}};
pub struct AnpaVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: String,
    pub build: String
}

impl AnpaVersion {
    fn new(major: u64, minor: u64, patch: u64, pre_release: impl Into<String>, build: impl Into<String>) -> AnpaVersion {
        AnpaVersion { major, minor, patch, pre_release: pre_release.into(), build: build.into() }
    }
}

pub fn version_parser<'a, S>() -> impl Parser<&'a str, AnpaVersion, S> {
    let number = integer_u64();
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

#[cfg(test)]
mod tests {

    use crate::{semver::version_parser, core::parse};

    #[test]
    fn version_no_snapshot() {
        let res = parse(version_parser(), "1.2.3").1.unwrap();
        assert_eq!(res.major, 1);
        assert_eq!(res.minor, 2);
        assert_eq!(res.patch, 3);
        assert!(res.pre_release.is_empty());
    }

    #[test]
    fn version_snapshot() {
        let res = parse(version_parser(), "12.345.67890-SNAPSHOT").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT".to_string());

        assert!(parse(version_parser(), "12.345.67890-").1.is_none());
        assert!(parse(version_parser(), "12.345.67890-+").1.is_none());
        assert!(parse(version_parser(), "12.345.67890-SNAPSHOT+").1.is_none());

    }

    #[test]
    fn version_build() {
        let res = parse(version_parser(), "12.345.67890+build1").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert!(res.pre_release.is_empty());
        assert_eq!(res.build, "build1".to_string());
    }

    #[test]
    fn version_build_and_snapshot() {
        let res = parse(version_parser(), "12.345.67890-SNAPSHOT+build1").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.pre_release, "SNAPSHOT".to_string());
        assert_eq!(res.build, "build1".to_string());
    }
}