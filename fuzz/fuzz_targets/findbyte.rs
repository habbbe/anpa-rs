#![no_main]
//! `find_byte`/`until_byte` (SWAR, 8 bytes per step) against a byte-at-a-time
//! reference that evaluates the same predicate with plain comparisons.
//!
//! Input: `[shape, k0, b0, k1, b1, k2, b2, flags, haystack...]`.

use anpa::core::parse;
use anpa::findbyte::{eq, find_byte, gt, lt, ne, until_byte, AndByte, ByteFinder, EqByte, GtByte, LtByte, NeByte, OrByte};
use libfuzzer_sys::fuzz_target;

#[path = "common.rs"]
mod common;
use common::known;

/// One of the four library finders, chosen at run time. Delegates to the
/// library's own implementation, so the SWAR arithmetic under test is anpa's.
#[derive(Clone, Copy)]
enum Leaf {
    Eq(EqByte),
    Ne(NeByte),
    Lt(LtByte),
    Gt(GtByte),
}

impl Leaf {
    fn new(kind: u8, b: u8) -> Leaf {
        match kind % 4 {
            0 => Leaf::Eq(eq(b)),
            1 => Leaf::Ne(ne(b)),
            2 => Leaf::Lt(lt(b)),
            _ => Leaf::Gt(gt(b)),
        }
    }

    /// A lane that borrows into the next one, and the value of that next
    /// lane that the borrow turns into a false positive of the intermediate
    /// (for `ne`, which inverts `eq`'s intermediate, a false negative).
    fn borrow_pair(self, x: u8, y: u8) -> bool {
        match self {
            Leaf::Eq(f) => x == f.0 && y == f.0 ^ 1,
            Leaf::Ne(f) => x == f.0 && y == f.0 ^ 1,
            Leaf::Lt(f) => x < f.0 && y == f.0,
            Leaf::Gt(f) => x > f.0 && y == f.0,
        }
    }

    fn reference(self, x: u8) -> bool {
        match self {
            Leaf::Eq(f) => x == f.0,
            Leaf::Ne(f) => x != f.0,
            Leaf::Lt(f) => x < f.0,
            Leaf::Gt(f) => x > f.0,
        }
    }
}

impl ByteFinder for Leaf {
    fn intermediate(self, haystack: usize) -> usize {
        match self {
            Leaf::Eq(f) => f.intermediate(haystack),
            Leaf::Ne(f) => f.intermediate(haystack),
            Leaf::Lt(f) => f.intermediate(haystack),
            Leaf::Gt(f) => f.intermediate(haystack),
        }
    }

    fn slow_cmp(self, other: u8) -> bool {
        match self {
            Leaf::Eq(f) => f.slow_cmp(other),
            Leaf::Ne(f) => f.slow_cmp(other),
            Leaf::Lt(f) => f.slow_cmp(other),
            Leaf::Gt(f) => f.slow_cmp(other),
        }
    }
}

/// With the `findbyte_lt_gt_range` category skipped, clamp the operands of
/// `lt`/`gt` (and, in `fuzz_target!`, the haystack bytes) to the ranges where
/// the SWAR tricks hold: `lt(n)` and `gt(n)` for n <= 127 over
/// haystack bytes below 0x80.
fn restrict(kind: u8, b: u8) -> u8 {
    if !known("findbyte_lt_gt_range") {
        return b;
    }
    match kind % 4 {
        2 => b.min(0x7f),       // lt(b): only b <= 127 (lt(128) loses lanes after a borrow)
        3 => b.min(0x7f),       // gt(b): only b <= 127
        _ => b,
    }
}

fn check<F: ByteFinder>(finder: F, reference: impl Fn(u8) -> bool, hay: &[u8], what: &str) {
    let expected = hay.iter().position(|&x| reference(x));

    for consume in [false, true] {
        let res = parse(find_byte(finder, consume), hay);
        match expected {
            None => {
                assert_eq!(res.result, None, "{what}: find_byte found a byte the reference does not; hay={hay:?}");
                assert_eq!(res.state, hay);
            }
            Some(pos) => {
                assert_eq!(res.result, Some(hay[pos]),
                           "{what}: find_byte result differs, expected pos {pos}; hay={hay:?}");
                assert_eq!(res.state, &hay[pos + consume as usize..],
                           "{what}: find_byte state differs, expected pos {pos}; hay={hay:?}");
            }
        }

        for include in [false, true] {
            let res = parse(until_byte(finder, include, consume), hay);
            match expected {
                None => assert_eq!(res.result, None, "{what}: until_byte; hay={hay:?}"),
                Some(pos) => {
                    assert_eq!(res.result, Some(&hay[..pos + include as usize]), "{what}: until_byte; hay={hay:?}");
                    assert_eq!(res.state, &hay[pos + consume as usize..], "{what}: until_byte; hay={hay:?}");
                }
            }
        }
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    let (head, hay) = data.split_at(8);
    let flags = head[7];
    let leaf = |i: usize| {
        let (k, b) = (head[1 + 2 * i], head[2 + 2 * i]);
        Leaf::new(k, restrict(k, b))
    };
    let (a, b, c) = (leaf(0), leaf(1), leaf(2));

    // Optionally misalign the haystack so the chunking starts mid-word.
    let hay = &hay[(flags as usize & 7).min(hay.len())..];
    let has_lt_gt = [1, 3, 5].iter().any(|&i| head[i] % 4 >= 2);
    let masked: Vec<u8>;
    let hay = if has_lt_gt && known("findbyte_lt_gt_range") {
        masked = hay.iter().map(|b| b & 0x7f).collect();
        &masked[..]
    } else {
        hay
    };

    if known("findbyte_borrow")
        && hay.windows(2).any(|w| [a, b, c].iter().any(|l| l.borrow_pair(w[0], w[1]))) {
        return;
    }
    let only_or = known("findbyte_and");
    match head[0] % 7 {
        0 => check(a, |x| a.reference(x), hay, "a"),
        1 => check(OrByte(a, b), |x| a.reference(x) || b.reference(x), hay, "a|b"),
        2 if !only_or => check(AndByte(a, b), |x| a.reference(x) && b.reference(x), hay, "a&b"),
        3 => check(OrByte(a, OrByte(b, c)), |x| a.reference(x) || b.reference(x) || c.reference(x), hay, "a|b|c"),
        4 if !only_or => check(AndByte(a, AndByte(b, c)), |x| a.reference(x) && b.reference(x) && c.reference(x), hay, "a&b&c"),
        5 if !only_or => check(OrByte(a, AndByte(b, c)), |x| a.reference(x) || (b.reference(x) && c.reference(x)), hay, "a|(b&c)"),
        6 if !only_or => check(AndByte(a, OrByte(b, c)), |x| a.reference(x) && (b.reference(x) || c.reference(x)), hay, "a&(b|c)"),
        _ => {}
    }
});
