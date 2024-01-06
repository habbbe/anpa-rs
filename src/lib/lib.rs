#[macro_use]
pub mod macros;
pub mod parsers;
pub mod core;
pub mod combinators;
pub mod slicelike;
pub mod version;

#[cfg(test)]
mod tests {

    use crate::{version::version_parser, core::parse};

    #[test]
    fn version_no_snapshot() {
        let res = parse(version_parser(), "1.2.3").1.unwrap();
        assert_eq!(res.major, 1);
        assert_eq!(res.minor, 2);
        assert_eq!(res.patch, 3);
        assert_eq!(res.snapshot, None);
    }

    #[test]
    fn version_snapshot() {
        let res = parse(version_parser(), "12.345.67890-SNAPSHOT").1.unwrap();
        assert_eq!(res.major, 12);
        assert_eq!(res.minor, 345);
        assert_eq!(res.patch, 67890);
        assert_eq!(res.snapshot, Some("SNAPSHOT".to_string()));
    }

    #[test]
    fn version_invalid_snapshot() {
        let res = parse(version_parser(), "1.2.3+SNAPSHOT").1;
        assert!(res.is_none());
    }
}