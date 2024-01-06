use crate::{parsers::{*}, core::{Parser, AnpaState}, combinators::{*}};

pub struct AnpaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub snapshot: Option<String>
}

pub fn version_parser<'a, S>() -> impl Parser<&'a str, AnpaVersion, S> {
    let component = left!(integer(), item('.'));
    let last_component = integer();
    let snapshot_valid = lift!(|s: &str| Some(s.to_string()), right(item('-'), rest()));
    let snapshot = or(snapshot_valid, lift!(|_| None, empty()));
    lift!(
        |major, minor, patch, snapshot| AnpaVersion { major, minor, patch, snapshot },
        component,
        component,
        last_component,
        snapshot)
}