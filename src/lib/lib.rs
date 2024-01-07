#[macro_use]
pub mod macros;
pub mod parsers;
pub mod core;
pub mod combinators;
pub mod slicelike;
pub mod version;

#[cfg(test)]
mod tests {

    use crate::{version::version_parser, core::parse, parsers::{integer_i8, integer_i8_checked}};

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

    #[test]
    fn signed_integer() {
        assert_eq!(parse(integer_i8(10), "0").1.unwrap(), 0);
        assert_eq!(parse(integer_i8(10), "127").1.unwrap(), 127);
        assert_eq!(parse(integer_i8(10), "-1").1.unwrap(), -1);
        assert_eq!(parse(integer_i8(10), "-128").1.unwrap(), -128);

        assert!(parse(integer_i8_checked(10), "-129").1.is_none());
        assert!(parse(integer_i8_checked(10), "128").1.is_none());
    }
}