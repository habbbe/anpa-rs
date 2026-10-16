#![no_main]
//! `take`/`skip`/`until` (via the `Prefix` and `Needle` impls) on `&str` and
//! `&[u8]`, with single-item and sequence arguments, against std.
//!
//! Input: `[mode, needle_len, needle..., haystack...]`.

use anpa::core::parse;
use anpa::parsers::{skip, take, until};
use libfuzzer_sys::fuzz_target;

#[path = "common.rs"]
mod common;
use common::known;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let (mode, n) = (data[0], data[1] as usize % 9);
    let rest = &data[2..];
    if rest.len() < n {
        return;
    }
    let (needle, hay) = rest.split_at(n);

    match mode % 4 {
        // &[u8] with a slice needle/prefix.
        0 => {
            if needle.is_empty() && known("needle_empty_slice") {
                return;
            }
            let r = parse(until(needle), hay);
            let pos = if needle.is_empty() { Some(0) } else { hay.windows(n).position(|w| w == needle) };
            assert_eq!(r.result, pos.map(|p| &hay[..p]));
            if let Some(p) = pos { assert_eq!(r.state, &hay[p + n..]); }

            let r = parse(take(needle), hay);
            assert_eq!(r.result, hay.starts_with(needle).then_some(needle));
            let r = parse(skip(needle), hay);
            assert_eq!(r.result.is_some(), hay.starts_with(needle));
        }
        // &[u8] with a single byte.
        1 => {
            let Some(&b) = needle.first() else { return };
            let r = parse(until(b), hay);
            let pos = hay.iter().position(|&x| x == b);
            assert_eq!(r.result, pos.map(|p| &hay[..p]));
            if let Some(p) = pos { assert_eq!(r.state, &hay[p + 1..]); }
            let r = parse(take(b), hay);
            assert_eq!(r.result, hay.first().filter(|&&x| x == b).copied());
        }
        // &str with a &str needle/prefix.
        2 => {
            let (Ok(needle), Ok(hay)) = (std::str::from_utf8(needle), std::str::from_utf8(hay)) else { return };
            let r = parse(until(needle), hay);
            let pos = hay.find(needle);
            assert_eq!(r.result, pos.map(|p| &hay[..p]));
            if let Some(p) = pos { assert_eq!(r.state, &hay[p + needle.len()..]); }
            let r = parse(take(needle), hay);
            assert_eq!(r.result, hay.starts_with(needle).then_some(needle));
        }
        // &str with a char.
        _ => {
            let (Ok(needle), Ok(hay)) = (std::str::from_utf8(needle), std::str::from_utf8(hay)) else { return };
            let Some(c) = needle.chars().next() else { return };
            let r = parse(until(c), hay);
            let pos = hay.find(c);
            assert_eq!(r.result, pos.map(|p| &hay[..p]));
            if let Some(p) = pos { assert_eq!(r.state, &hay[p + c.len_utf8()..]); }
            let r = parse(take(c), hay);
            assert_eq!(r.result, hay.starts_with(c).then_some(c));
            let r = parse(skip(c), hay);
            assert_eq!(r.state, hay.strip_prefix(c).unwrap_or(hay));
        }
    }
});
