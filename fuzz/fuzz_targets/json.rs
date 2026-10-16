#![no_main]
//! `json::value_parser` on arbitrary UTF-8, with `serde_json` as the oracle.
//!
//! - never panics, hangs or overflows the stack;
//! - **soundness:** a document anpa accepts in full (only whitespace left) is
//!   valid JSON to serde_json, and the two values agree (strings unescaped,
//!   numbers within a tolerance);
//! - **completeness, for documents without numbers:** a document serde_json
//!   accepts that holds no number is accepted by anpa (anpa's numbers have no
//!   exponent and fit `isize.usize`, so they are left out of this direction).

use std::collections::BTreeMap;

use anpa::core::parse;
use anpa::json::{value_parser, JsonValue};
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

#[path = "common.rs"]
mod common;
use common::known;

/// Unescape a JSON string body as anpa returned it (raw, between the quotes).
/// `None` on a lone surrogate, which serde_json rejects.
fn unescape(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next()? {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            '/' => out.push('/'),
            'b' => out.push('\u{8}'),
            'f' => out.push('\u{c}'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            'u' => {
                let hex = |it: &mut std::str::Chars| -> Option<u32> {
                    let s: String = it.by_ref().take(4).collect();
                    u32::from_str_radix(&s, 16).ok()
                };
                let hi = hex(&mut it)?;
                if (0xD800..0xDC00).contains(&hi) {
                    if it.next()? != '\\' || it.next()? != 'u' {
                        return None;
                    }
                    let lo = hex(&mut it)?;
                    if !(0xDC00..0xE000).contains(&lo) {
                        return None;
                    }
                    out.push(char::from_u32(0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00))?);
                } else {
                    out.push(char::from_u32(hi)?);
                }
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Compare anpa's value with serde_json's. `Err` names the first difference.
fn same(a: &JsonValue<&str>, s: &Value) -> Result<(), String> {
    match (a, s) {
        (JsonValue::Null, Value::Null) => Ok(()),
        (JsonValue::Bool(x), Value::Bool(y)) if x == y => Ok(()),
        (JsonValue::Str(x), Value::String(y)) => {
            let u = unescape(x).ok_or_else(|| format!("anpa string {x:?} does not unescape"))?;
            (u == *y).then_some(()).ok_or_else(|| format!("string {u:?} vs {y:?}"))
        }
        (JsonValue::Num(x), Value::Number(n)) => {
            let y = n.as_f64().unwrap();
            let close = *x == y || (x - y).abs() <= 1e-12 * x.abs().max(y.abs());
            close.then_some(()).ok_or_else(|| format!("number {x:?} vs {y:?}"))
        }
        (JsonValue::Arr(xs), Value::Array(ys)) => {
            if xs.len() != ys.len() {
                return Err(format!("array length {} vs {}", xs.len(), ys.len()));
            }
            xs.iter().zip(ys).try_for_each(|(x, y)| same(x, y))
        }
        (JsonValue::Dic(xs), Value::Object(ys)) => {
            // anpa keys are raw (escaped); serde's are unescaped. Two raw keys
            // that unescape to the same key make the comparison ambiguous.
            let mut un: BTreeMap<String, &JsonValue<&str>> = BTreeMap::new();
            for (k, v) in xs {
                let k = unescape(k).ok_or_else(|| format!("anpa key {k:?} does not unescape"))?;
                if un.insert(k, v).is_some() {
                    return Ok(());
                }
            }
            if un.len() != ys.len() {
                return Err(format!("object size {} vs {}", un.len(), ys.len()));
            }
            for (k, v) in &un {
                let y = ys.get(k).ok_or_else(|| format!("key {k:?} missing in serde"))?;
                same(v, y)?;
            }
            Ok(())
        }
        _ => Err(format!("kind {a:?} vs {s}")),
    }
}

/// A digit or `-` outside a string: a number token. (Not from serde's value,
/// where a duplicate key can replace a number.)
fn has_number(input: &str) -> bool {
    let mut in_str = false;
    let mut esc = false;
    for c in input.bytes() {
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (false, b'\\') => esc = true,
                (false, b'"') => in_str = false,
                _ => {}
            }
        } else if c == b'"' {
            in_str = true;
        } else if c.is_ascii_digit() || c == b'-' {
            return true;
        }
    }
    false
}

/// Inputs in documented divergence categories (see fuzz-test.md).
fn known_divergence(input: &str) -> bool {
    // A number with a leading zero, which JSON forbids (anpa's float allows).
    let b = input.as_bytes();
    let leading_zero = (0..b.len().saturating_sub(1)).any(|i| {
        b[i] == b'0' && b[i + 1].is_ascii_digit() && (i == 0 || !b[i - 1].is_ascii_digit() && b[i - 1] != b'.')
    });
    (leading_zero && known("json_leading_zero"))
        || (input.contains('\u{c}') && known("json_formfeed_ws"))
        || ((input.contains("\\ud") || input.contains("\\uD")) && known("json_lone_surrogate"))
}

/// A model of anpa's JSON grammar, combinator by combinator, including where
/// input is kept on failure: `or` restores it before its second parser, but
/// `many` keeps whatever a failed element consumed. `leaked` records when an
/// element of a successful `many` failed after consuming more than whitespace
/// (`{"k"}`, `[{"a" ]`, `"\u12"`), which is where anpa accepts invalid JSON.
#[derive(Debug, PartialEq)]
enum M<'a> {
    Null,
    Bool(bool),
    Str(&'a str),
    Num(u64),
    Arr(Vec<M<'a>>),
    Dic(BTreeMap<&'a str, M<'a>>),
}

struct Model<'a> {
    s: &'a str,
    b: &'a [u8],
    pos: usize,
    leaked: bool,
}

impl<'a> Model<'a> {
    fn ws(&mut self) {
        while self.pos < self.b.len() && self.b[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn skip(&mut self, c: u8) -> bool {
        let ok = self.b.get(self.pos) == Some(&c);
        self.pos += ok as usize;
        ok
    }

    fn literal(&mut self, lit: &str) -> bool {
        let ok = self.s[self.pos..].starts_with(lit);
        if ok {
            self.pos += lit.len();
        }
        ok
    }

    /// Note a failed element of a successful `many` that consumed past `from`
    /// more than whitespace.
    fn leak(&mut self, from: usize) {
        if !self.s[from..self.pos].trim_ascii_start().is_empty() {
            self.leaked = true;
        }
    }

    fn value(&mut self) -> Option<M<'a>> {
        self.ws();
        let start = self.pos;
        // `or!` restores the input before each alternative.
        if let Some(v) = self.string().map(M::Str) { return Some(v) }
        self.pos = start;
        if let Some(v) = self.number() { return Some(v) }
        self.pos = start;
        if let Some(v) = self.object() { return Some(v) }
        self.pos = start;
        if let Some(v) = self.array() { return Some(v) }
        self.pos = start;
        if self.literal("true") { return Some(M::Bool(true)) }
        if self.literal("false") { return Some(M::Bool(false)) }
        if self.literal("null") { return Some(M::Null) }
        None
    }

    fn string(&mut self) -> Option<&'a str> {
        if !self.skip(b'"') {
            return None;
        }
        let body = self.pos;
        loop {
            // find_byte(eq('"') | eq('\\') | lt(0x20), false): fails without
            // consuming when there is none.
            let Some(q) = self.b[self.pos..].iter().position(|&c| c == b'"' || c == b'\\' || c < 0x20) else { break };
            self.pos += q;
            if self.b[self.pos] != b'\\' {
                break;
            }
            let el = self.pos;
            self.pos += 1; // item(): the backslash
            match self.b.get(self.pos) {
                Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => self.pos += 1,
                Some(b'u') => {
                    self.pos += 1;
                    let hex = self.b[self.pos..].iter().take(4).take_while(|c| c.is_ascii_hexdigit()).count();
                    self.pos += hex;
                    if hex < 4 {
                        self.leak(el);
                        break;
                    }
                }
                _ => {
                    self.leak(el);
                    break;
                }
            }
        }
        let raw = &self.s[body..self.pos];
        self.skip(b'"').then_some(raw)
    }

    fn number(&mut self) -> Option<M<'a>> {
        // float(): an isize integer part, then optionally '.' and an unsigned
        // usize with a usize divisor 10^n (falling back on overflow).
        let t = &self.b[self.pos..];
        let neg = t.first() == Some(&b'-');
        let i = neg as usize;
        let n = t[i..].iter().take_while(|c| c.is_ascii_digit()).count();
        if n == 0 {
            return None;
        }
        let int: isize = self.s[self.pos..self.pos + i + n].parse().ok()?;
        let mut end = i + n;
        let mut v = int as f64;
        if t.get(end) == Some(&b'.') {
            let d = t[end + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
            let ds = &self.s[self.pos + end + 1..self.pos + end + 1 + d];
            if let (true, Ok(dec), Some(div)) = (d > 0, ds.parse::<usize>(), 10usize.checked_pow(d as u32)) {
                v = int as f64 + if neg { -1.0 } else { 1.0 } * (dec as f64) / (div as f64);
                end += 1 + d;
            }
        }
        self.pos += end;
        Some(M::Num(v.to_bits()))
    }

    /// `many` with `separator(eat(','), false)`, allowing empty.
    fn many<T>(&mut self, mut el: impl FnMut(&mut Self) -> Option<T>) -> Option<Vec<T>> {
        let mut out = vec![];
        loop {
            let from = self.pos;
            match el(self) {
                Some(x) => out.push(x),
                None => {
                    if out.is_empty() {
                        self.leak(from);
                        return Some(out);
                    }
                    return None; // after a separator, with no trailing one allowed
                }
            }
            self.ws();
            if !self.skip(b',') {
                return Some(out);
            }
        }
    }

    fn object(&mut self) -> Option<M<'a>> {
        if !self.skip(b'{') {
            return None;
        }
        let pairs = self.many(|m| {
            m.ws();
            let k = m.string()?;
            m.ws();
            if !m.skip(b':') {
                return None;
            }
            Some((k, m.value()?))
        })?;
        self.ws();
        self.skip(b'}').then(|| M::Dic(pairs.into_iter().collect()))
    }

    fn array(&mut self) -> Option<M<'a>> {
        if !self.skip(b'[') {
            return None;
        }
        let xs = self.many(|m| m.value())?;
        self.ws();
        self.skip(b']').then_some(M::Arr(xs))
    }
}

fn to_model<'a>(v: &JsonValue<&'a str>) -> M<'a> {
    match v {
        JsonValue::Null => M::Null,
        JsonValue::Bool(b) => M::Bool(*b),
        JsonValue::Str(s) => M::Str(s),
        JsonValue::Num(n) => M::Num(n.to_bits()),
        JsonValue::Arr(xs) => M::Arr(xs.iter().map(to_model).collect()),
        JsonValue::Dic(m) => M::Dic(m.iter().map(|(k, v)| (*k, to_model(v))).collect()),
    }
}

/// Nesting depth, as an upper bound of the parser's recursion.
fn depth(input: &str) -> usize {
    let (mut d, mut max) = (0usize, 0usize);
    for c in input.bytes() {
        match c {
            b'[' | b'{' => { d += 1; max = max.max(d); }
            b']' | b'}' => d = d.saturating_sub(1),
            _ => {}
        }
    }
    max
}

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else { return };
    if known("json_deep_nesting") && depth(input) > 64 {
        return;
    }

    let res = parse(value_parser::<&str>(), input);
    let full = res.result.is_some() && res.state.trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r')).is_empty();
    let serde = serde_json::from_str::<Value>(input);

    // anpa against the model of its own grammar: exact, always on.
    let mut m = Model { s: input, b: input.as_bytes(), pos: 0, leaked: false };
    let mv = m.value();
    assert_eq!(res.result.as_ref().map(to_model), mv, "anpa differs from its grammar model: {input:?}");
    if mv.is_some() {
        assert_eq!(res.state, &input[m.pos..], "anpa's remaining input differs from the model: {input:?}");
    }

    if known_divergence(input) || (m.leaked && known("json_many_partial")) {
        return;
    }

    if full {
        let v = serde.as_ref().unwrap_or_else(|e| panic!("anpa accepts, serde_json rejects ({e}): {input:?}"));
        if let Err(diff) = same(res.result.as_ref().unwrap(), v) {
            panic!("values differ ({diff}): {input:?}");
        }
    } else if serde.is_ok() {
        if !has_number(input) {
            panic!("serde_json accepts a number-free document anpa rejects: {input:?} (left: {:?})", res.state);
        }
    }
});
