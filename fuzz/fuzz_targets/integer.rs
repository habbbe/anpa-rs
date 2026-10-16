#![no_main]
//! `integer`/`integer_custom` (checked configs, every integer type) against a
//! reference built on `from_str_radix`, over both `&[u8]` and `&str` input;
//! and the `raw_{le,be,ne}_integer` parsers against `from_*_bytes`.
//!
//! Input: `[type, config, mode, text...]`.

use anpa::core::parse;
use anpa::number::{integer_custom, raw_be_integer, raw_le_integer, raw_ne_integer, IntConfig, NumLike};
use libfuzzer_sys::fuzz_target;

#[path = "common.rs"]
mod common;
use common::known;

#[derive(Clone, Copy)]
struct Cfg {
    unsigned: bool,
    leading_plus: bool,
    leading_zeros: bool,
    base: u32,
}

/// What anpa should return: `Some((value as i128 or u128 text, consumed bytes))`.
fn reference<T: std::str::FromStr + FromRadix>(text: &[u8], signed_type: bool, cfg: Cfg) -> Option<(T, usize)> {
    let signed = signed_type && !cfg.unsigned;
    let mut i = 0;
    let mut neg = false;
    if let Some(&c) = text.first() {
        if signed && c == b'-' {
            neg = true;
            i = 1;
        } else if cfg.leading_plus && c == b'+' {
            i = 1;
        }
    }
    let start = i;
    while i < text.len() && (text[i] as char).to_digit(cfg.base).is_some() {
        i += 1;
    }
    let digits = &text[start..i];
    if digits.is_empty() {
        return None;
    }
    if !cfg.leading_zeros && digits.len() > 1 && digits[0] == b'0' {
        return None;
    }
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    s.push_str(std::str::from_utf8(digits).unwrap());
    T::from_radix(&s, cfg.base).map(|v| (v, i))
}

trait FromRadix: Sized {
    fn from_radix(s: &str, base: u32) -> Option<Self>;
}

macro_rules! from_radix {
    ($($t:ty),*) => {$(
        impl FromRadix for $t {
            fn from_radix(s: &str, base: u32) -> Option<Self> {
                <$t>::from_str_radix(s, base).ok()
            }
        }
    )*};
}
from_radix!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

fn check_one<T, const SIGNED: u8, const PLUS: bool, const LZ: bool, const BASE: u8>(
    config: IntConfig<true, SIGNED, PLUS, LZ, BASE>,
    text: &[u8],
    cfg: Cfg,
) where
    T: NumLike + FromRadix + std::str::FromStr + std::fmt::Debug + PartialEq,
{
    let signed_type = T::ZERO > T::MIN;
    let expected = reference::<T>(text, signed_type, cfg);

    let res = parse(integer_custom::<true, SIGNED, PLUS, LZ, BASE, T, _, _, ()>(config), text);
    match expected {
        Some((v, used)) => {
            assert_eq!(res.result, Some(v), "bytes {:?} cfg unsigned={} plus={} lz={} base={}",
                       String::from_utf8_lossy(text), cfg.unsigned, cfg.leading_plus, cfg.leading_zeros, cfg.base);
            assert_eq!(res.state, &text[used..]);
        }
        None => assert_eq!(res.result, None, "bytes {:?} cfg unsigned={} plus={} lz={} base={}",
                           String::from_utf8_lossy(text), cfg.unsigned, cfg.leading_plus, cfg.leading_zeros, cfg.base),
    }

    if let Ok(s) = std::str::from_utf8(text) {
        let res = parse(integer_custom::<true, SIGNED, PLUS, LZ, BASE, T, _, _, ()>(config), s);
        match expected {
            Some((v, used)) => {
                assert_eq!(res.result, Some(v), "str {s:?}");
                assert_eq!(res.state, &s[used..]);
            }
            None => assert_eq!(res.result, None, "str {s:?}"),
        }
    }
}

fn check_configs<T>(cfg_sel: u8, text: &[u8])
where
    T: NumLike + FromRadix + std::str::FromStr + std::fmt::Debug + PartialEq,
{
    let c = |unsigned, leading_plus, leading_zeros, base| Cfg { unsigned, leading_plus, leading_zeros, base };
    match cfg_sel % 10 {
        0 => check_one::<T, _, _, _, _>(IntConfig::new(), text, c(false, false, true, 10)),
        1 => check_one::<T, _, _, _, _>(IntConfig::new().unsigned(), text, c(true, false, true, 10)),
        2 => check_one::<T, _, _, _, _>(IntConfig::new().leading_plus(), text, c(false, true, true, 10)),
        3 => check_one::<T, _, _, _, _>(IntConfig::new().no_leading_zero(), text, c(false, false, false, 10)),
        4 => check_one::<T, _, _, _, _>(IntConfig::new().base::<16>(), text, c(false, false, true, 16)),
        5 => check_one::<T, _, _, _, _>(IntConfig::new().base::<2>(), text, c(false, false, true, 2)),
        6 => check_one::<T, _, _, _, _>(IntConfig::new().base::<36>(), text, c(false, false, true, 36)),
        7 => check_one::<T, _, _, _, _>(IntConfig::new().unsigned().leading_plus(), text, c(true, true, true, 10)),
        8 => check_one::<T, _, _, _, _>(IntConfig::new().leading_plus().no_leading_zero().base::<16>(),
                                          text, c(false, true, false, 16)),
        _ => check_one::<T, _, _, _, _>(IntConfig::new().unsigned().no_leading_zero().base::<8>(),
                                          text, c(true, false, false, 8)),
    }
}

macro_rules! check_raw {
    ($t:ty, $bytes:expr) => {{
        let bytes: &[u8] = $bytes;
        const N: usize = std::mem::size_of::<$t>();
        let exact_only = known("raw_integer_exact_len");
        if !exact_only || bytes.len() == N {
            let first: Option<[u8; N]> = bytes.get(..N).map(|b| b.try_into().unwrap());
            let r = parse(raw_le_integer::<$t, ()>(), bytes);
            assert_eq!(r.result, first.map(<$t>::from_le_bytes), "raw_le {} len {}", stringify!($t), bytes.len());
            if first.is_some() { assert_eq!(r.state, &bytes[N..]); }
            let r = parse(raw_be_integer::<$t, ()>(), bytes);
            assert_eq!(r.result, first.map(<$t>::from_be_bytes), "raw_be {} len {}", stringify!($t), bytes.len());
            if first.is_some() { assert_eq!(r.state, &bytes[N..]); }
            let r = parse(raw_ne_integer::<$t, ()>(), bytes);
            assert_eq!(r.result, first.map(<$t>::from_ne_bytes), "raw_ne {} len {}", stringify!($t), bytes.len());
            if first.is_some() { assert_eq!(r.state, &bytes[N..]); }
        }
    }};
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }
    let (ty, cfg, mode, text) = (data[0], data[1], data[2], &data[3..]);

    if mode % 4 == 0 {
        match ty % 12 {
            0 => check_raw!(u8, text),
            1 => check_raw!(i8, text),
            2 => check_raw!(u16, text),
            3 => check_raw!(i16, text),
            4 => check_raw!(u32, text),
            5 => check_raw!(i32, text),
            6 => check_raw!(u64, text),
            7 => check_raw!(i64, text),
            8 => check_raw!(u128, text),
            9 => check_raw!(i128, text),
            10 => check_raw!(usize, text),
            _ => check_raw!(isize, text),
        }
        return;
    }

    match ty % 12 {
        0 => check_configs::<u8>(cfg, text),
        1 => check_configs::<i8>(cfg, text),
        2 => check_configs::<u16>(cfg, text),
        3 => check_configs::<i16>(cfg, text),
        4 => check_configs::<u32>(cfg, text),
        5 => check_configs::<i32>(cfg, text),
        6 => check_configs::<u64>(cfg, text),
        7 => check_configs::<i64>(cfg, text),
        8 => check_configs::<u128>(cfg, text),
        9 => check_configs::<i128>(cfg, text),
        10 => check_configs::<usize>(cfg, text),
        _ => check_configs::<isize>(cfg, text),
    }
});
