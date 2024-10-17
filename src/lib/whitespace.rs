use crate::{asciilike::AsciiLike, core::Parser, prefix::Prefix};

/// Trait for inputs that can be trimmed, i.e. having ASCII whitespace removed
/// from the start.
pub trait TrimmableAscii: Copy {
    fn prefix() -> impl Prefix<Self, ()>;
}

impl<'a, A: AsciiLike> TrimmableAscii for &'a [A] {
    fn prefix() -> impl Prefix<Self, ()> {
        IgnoreAsciiWhitespace()
    }
}

impl<'a> TrimmableAscii for &'a str {
    #[inline(always)]
    fn prefix() -> impl Prefix<Self, ()> {
        IgnoreAsciiWhitespace()
    }
}

/// Create a parser that parses and ignores whitespace.
#[inline]
pub fn ignore_ascii_whitespace<I: TrimmableAscii, S>() -> impl Parser<I, (), S> {
    elem!(I::prefix())
}

/// `Prefix` that consumes and ignores ASCII whitespaces.
#[derive(Clone, Copy)]
pub struct IgnoreAsciiWhitespace();

impl<'a, A: AsciiLike + Copy> Prefix<&'a [A], ()> for IgnoreAsciiWhitespace {
    fn remove_prefix(&self, haystack: &'a [A]) -> Option<((), &'a [A])> {
        let idx = haystack.iter().position(|a| !a.is_whitespace_ascii()).unwrap_or(haystack.len());
        Some(((), &haystack[idx..]))
    }
}

impl<'a> Prefix<&'a str, ()> for IgnoreAsciiWhitespace {
    #[inline]
    fn remove_prefix(&self, haystack: &'a str) -> Option<((), &'a str)> {
        Some(((), haystack.trim_ascii_start()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, whitespace::ignore_ascii_whitespace};

    #[test]
    fn test_whitespace_ignore_u8() {
        let input = &[b' ', b' ', 1, 2];
        let res = parse(ignore_ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &[1, 2]);
    }

    #[test]
    fn test_whitespace_ignore_u8_nothing() {
        let input = &[1, 2];
        let res = parse(ignore_ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &[1, 2]);
    }

    #[test]
    fn test_whitespace_ignore_char() {
        let input = &[' ', ' ', '1', '2'];
        let res = parse(ignore_ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &['1', '2']);
    }

    #[test]
    fn test_whitespace_ignore_str() {
        let input = "  12";
        let res = parse(ignore_ascii_whitespace(), input);
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, "12");
    }
}