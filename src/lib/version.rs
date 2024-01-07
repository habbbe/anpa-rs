use crate::{parsers::{*}, core::{Parser, AnpaState}, combinators::{*}};

pub struct AnpaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub snapshot: Option<String>
}

pub fn version_parser<'a, S>() -> impl Parser<&'a str, AnpaVersion, S> {
    let component = left!(integer_u32(10), item('.'));
    let last_component = integer_u32(10);
    let snapshot_valid = lift!(|s: &str| Some(s.to_string()), right(item('-'), rest()));
    let snapshot = or(snapshot_valid, lift!(|_| None, empty()));
    lift!(
        |major, minor, patch, snapshot| AnpaVersion { major, minor, patch, snapshot },
        component,
        component,
        last_component,
        snapshot)
}

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
}