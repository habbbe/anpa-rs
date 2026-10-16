#![no_main]
//! `float`/`float_custom` (checked configs, `f32` and `f64`) against a model
//! of anpa's grammar for what is consumed, and std's correctly rounded
//! `str::parse` for the value (with a tolerance: anpa computes
//! `int + dec / 10^n`, scaled by `10.powi(exp)`, which is not correctly rounded).
//!
//! Input: `[type, config, text...]`.

use anpa::core::parse;
use anpa::number::{float_custom, FloatConfig, FloatLike};
use libfuzzer_sys::fuzz_target;

#[path = "common.rs"]
mod common;
use common::known;

#[derive(Clone, Copy, Debug)]
struct Cfg {
    unsigned: bool,
    sci: bool,
    leading_plus: bool,
    lz_int: bool,
    lz_exp: bool,
    comma: bool,
}

/// Greedy run of ASCII digits from `i`.
fn digits(t: &[u8], i: usize) -> usize {
    t[i..].iter().take_while(|c| c.is_ascii_digit()).count()
}

/// Anpa's integer grammar on a checked type: optional sign, digits; `None` on
/// no digits, a forbidden leading zero or overflow. Returns the end offset.
fn int_part(t: &[u8], signed: bool, plus: bool, lz: bool, fits: impl Fn(bool, &str) -> bool) -> Option<usize> {
    let mut i = 0;
    let mut neg = false;
    match t.first() {
        Some(b'-') if signed => { neg = true; i = 1; }
        Some(b'+') if plus => i = 1,
        _ => {}
    }
    let n = digits(t, i);
    if n == 0 || (!lz && n > 1 && t[i] == b'0') {
        return None;
    }
    let ds = std::str::from_utf8(&t[i..i + n]).unwrap();
    fits(neg, ds).then_some(i + n)
}

/// The consumed length and the text std should parse, per anpa's grammar.
fn model(t: &[u8], cfg: Cfg) -> Option<(usize, String, Option<i64>)> {
    let int_end = int_part(t, !cfg.unsigned, cfg.leading_plus, cfg.lz_int, |neg, ds| {
        let s = if neg { format!("-{ds}") } else { ds.to_owned() };
        s.parse::<isize>().is_ok()
    })?;
    let mut end = int_end;
    let sep = if cfg.comma { b',' } else { b'.' };
    let mut text = String::from_utf8(t[..int_end].to_vec()).unwrap();

    // Decimal part: an unsigned usize with a usize divisor 10^n; any
    // overflow fails it, and the parse falls back to the integer part.
    if t.get(end) == Some(&sep) {
        let n = digits(t, end + 1);
        let ds = std::str::from_utf8(&t[end + 1..end + 1 + n]).unwrap();
        if n > 0 && ds.parse::<usize>().is_ok() && 10usize.checked_pow(n as u32).is_some() {
            text.push('.');
            text.push_str(ds);
            end += 1 + n;
        }
    }

    let mut exp = None;
    if cfg.sci && matches!(t.get(end), Some(b'e' | b'E')) {
        let rest = &t[end + 1..];
        if let Some(e_end) = int_part(rest, true, true, cfg.lz_exp, |neg, ds| {
            let s = if neg { format!("-{ds}") } else { ds.to_owned() };
            s.parse::<i32>().is_ok()
        }) {
            let es = std::str::from_utf8(&rest[..e_end]).unwrap();
            exp = Some(es.parse::<i64>().unwrap());
            text.push('e');
            text.push_str(es);
            end += 1 + e_end;
        }
    }
    Some((end, text, exp))
}

trait Float: FloatLike + std::str::FromStr + std::fmt::Debug + PartialEq + Copy {
    const TOL: f64;
    const MAX_EXP10: i64;
    fn to_f64(self) -> f64;
    fn is_normal_or_zero(self) -> bool;
}

impl Float for f32 {
    const TOL: f64 = 1e-5;
    const MAX_EXP10: i64 = 37;
    fn to_f64(self) -> f64 { self as f64 }
    fn is_normal_or_zero(self) -> bool { self == 0.0 || self.is_normal() }
}

impl Float for f64 {
    const TOL: f64 = 1e-12;
    const MAX_EXP10: i64 = 307;
    fn to_f64(self) -> f64 { self }
    fn is_normal_or_zero(self) -> bool { self == 0.0 || self.is_normal() }
}

fn check_one<T: Float, const SIGNED: u8, const SCI: bool, const PLUS: bool, const LZI: bool, const LZE: bool, const COMMA: bool>(
    config: FloatConfig<true, SIGNED, SCI, PLUS, LZI, LZE, COMMA>,
    t: &[u8],
    cfg: Cfg,
) {
    let expected = model(t, cfg);
    let res = parse(float_custom::<true, SIGNED, SCI, PLUS, LZI, LZE, COMMA, T, _, _, _, ()>(config), t);

    let Some((used, text, exp)) = expected else {
        assert_eq!(res.result, None, "input {:?} {cfg:?}", String::from_utf8_lossy(t));
        return;
    };
    let got = res.result.unwrap_or_else(|| panic!("anpa rejected {:?} {cfg:?}", String::from_utf8_lossy(t)));
    assert_eq!(res.state, &t[used..], "consumed differs on {:?} {cfg:?}", String::from_utf8_lossy(t));

    let want: T = text.parse().ok().unwrap();
    let (g, w) = (got.to_f64(), want.to_f64());

    // 10^exp, or the result, outside the normal range: anpa multiplies by
    // `10.powi(exp)`, which over/underflows before the mantissa is applied.
    let out_of_range = exp.is_some_and(|e| e.abs() > T::MAX_EXP10) || !want.is_normal_or_zero() || !got.is_normal_or_zero();
    if out_of_range && known("float_exp_range") {
        return;
    }

    let close = g == w || ((g - w).abs() <= T::TOL * w.abs().max(g.abs()));
    assert!(close, "value differs on {text:?}: anpa {got:?}, std {want:?} {cfg:?}");

    if let Ok(s) = std::str::from_utf8(t) {
        let res2 = parse(float_custom::<true, SIGNED, SCI, PLUS, LZI, LZE, COMMA, T, _, _, _, ()>(config), s);
        assert_eq!(res2.result.map(|v: T| v.to_f64().to_bits()), Some(g.to_bits()), "str/bytes differ on {s:?}");
        assert_eq!(res2.state.len(), t.len() - used);
    }
}

fn check_configs<T: Float>(sel: u8, t: &[u8]) {
    let c = |unsigned, sci, leading_plus, lz_int, lz_exp, comma| Cfg { unsigned, sci, leading_plus, lz_int, lz_exp, comma };
    let f = FloatConfig::new();
    match sel % 8 {
        0 => check_one::<T, _, _, _, _, _, _>(f, t, c(false, false, false, true, true, false)),
        1 => check_one::<T, _, _, _, _, _, _>(f.scientific(), t, c(false, true, false, true, true, false)),
        2 => check_one::<T, _, _, _, _, _, _>(f.leading_plus(), t, c(false, false, true, true, true, false)),
        3 => check_one::<T, _, _, _, _, _, _>(f.unsigned(), t, c(true, false, false, true, true, false)),
        4 => check_one::<T, _, _, _, _, _, _>(f.no_leading_zero_int(), t, c(false, false, false, false, true, false)),
        5 => check_one::<T, _, _, _, _, _, _>(f.decimal_comma(), t, c(false, false, false, true, true, true)),
        6 => check_one::<T, _, _, _, _, _, _>(f.scientific().no_leading_zero_exp(), t, c(false, true, false, true, false, false)),
        _ => check_one::<T, _, _, _, _, _, _>(f.scientific().leading_plus().no_leading_zero_int().decimal_comma(),
                                              t, c(false, true, true, false, true, true)),
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let (ty, cfg, text) = (data[0], data[1], &data[2..]);
    if ty % 2 == 0 {
        check_configs::<f32>(cfg, text);
    } else {
        check_configs::<f64>(cfg, text);
    }
});
