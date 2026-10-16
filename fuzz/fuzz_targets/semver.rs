#![no_main]
//! `anpa::semver::parse_inline` against the `semver` crate (SemVer 2.0.0):
//! both accept or both reject, and accepted versions agree field by field.

use libfuzzer_sys::fuzz_target;

#[path = "common.rs"]
mod common;
use common::known;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else { return };
    let ours = anpa::semver::parse_inline(input);
    let theirs = semver::Version::parse(input);

    match (&ours, &theirs) {
        (Some(a), Ok(b)) => {
            assert_eq!((a.major, a.minor, a.patch), (b.major, b.minor, b.patch), "{input:?}");
            assert_eq!(a.pre_release, b.pre.as_str(), "{input:?}");
            assert_eq!(a.build, b.build.as_str(), "{input:?}");
        }
        (None, Err(_)) => {}
        (Some(a), Err(e)) => {
            // A version core component with leading zeros, e.g. `00.1.2`.
            let core = input.split(['-', '+']).next().unwrap_or("");
            let lz = core.split('.').any(|c| c.len() > 1 && c.starts_with('0'));
            if lz && known("semver_core_leading_zero") {
                return;
            }
            panic!("anpa accepts, semver rejects ({e}): {input:?} -> {a:?}");
        }
        (None, Ok(b)) => panic!("semver accepts, anpa rejects: {input:?} -> {b:?}"),
    }
});
