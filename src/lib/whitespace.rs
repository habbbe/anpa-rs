use crate::{charlike::CharLike, core::Parser, prefix::Prefix};

/// Trait for inputs that can be trimmed, i.e. having ASCII whitespace removed
/// from the start.
pub trait TrimmableAscii: Copy {
    fn prefix() -> impl Prefix<Self, Self>;
}

impl<A: CharLike> TrimmableAscii for &[A] {
    fn prefix() -> impl Prefix<Self, Self> {
        AsciiWhitespace()
    }
}

impl TrimmableAscii for &str {
    fn prefix() -> impl Prefix<Self, Self> {
        AsciiWhitespace()
    }
}

/// Create a parser that parses and returns ASCII whitespace.
#[inline]
pub fn ascii_whitespace<I: TrimmableAscii, S>() -> impl Parser<I, I, S> {
    take!(I::prefix())
}

/// Create a parser that parses and ignores ASCII whitespace.
#[inline]
pub fn skip_ascii_whitespace<I: TrimmableAscii, S>() -> impl Parser<I, (), S> {
    skip!(I::prefix())
}

/// `Prefix` that matches zero or more ASCII whitespaces.
#[derive(Clone, Copy)]
pub struct AsciiWhitespace();

impl AsciiWhitespace {
    fn count_whitespace<A: CharLike>(slice: &[A]) -> usize {
        slice.iter().position(|a| !a.as_char().is_ascii_whitespace()).unwrap_or(slice.len())
    }
}

impl<'a, A: CharLike> Prefix<&'a [A], &'a [A]> for AsciiWhitespace {
    fn take_prefix(&self, haystack: &'a [A]) -> Option<(&'a [A], &'a [A])> {
        let idx = Self::count_whitespace(haystack);
        Some(haystack.split_at(idx))
    }

    fn skip_prefix(&self, haystack: &'a [A]) -> Option<&'a [A]> {
        Some(&haystack[Self::count_whitespace(haystack)..])
    }
}

impl<'a> Prefix<&'a str, &'a str> for AsciiWhitespace {
    fn take_prefix(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
        let trimmed = haystack.trim_ascii_start();
        Some((&haystack[..haystack.len() - trimmed.len()], trimmed))
    }

    fn skip_prefix(&self, haystack: &'a str) -> Option<&'a str> {
        Some(haystack.trim_ascii_start())
    }
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, whitespace::{ascii_whitespace, skip_ascii_whitespace}};

    #[test]
    fn test_whitespace_u8() {
        let input = &[b' ', b' ', 1, 2];
        let res = parse(skip_ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &[1, 2]);

        let res = parse(ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some([b' ', b' '].as_slice()));
        assert_eq!(res.state, &[1, 2]);
    }

    #[test]
    fn test_whitespace_u8_nothing() {
        let input = &[1, 2];
        let res = parse(skip_ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &[1, 2]);

        let res = parse(ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some([].as_slice()));
        assert_eq!(res.state, &[1, 2]);
    }

    #[test]
    fn test_whitespace_char() {
        let input = &[' ', ' ', '1', '2'];
        let res = parse(skip_ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &['1', '2']);

        let res = parse(ascii_whitespace(), input.as_slice());
        assert_eq!(res.result, Some([' ', ' '].as_slice()));
        assert_eq!(res.state, &['1', '2']);
    }

    #[test]
    fn test_whitespace_str() {
        let input = "  12";
        let res = parse(skip_ascii_whitespace(), input);
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, "12");

        let res = parse(ascii_whitespace(), input);
        assert_eq!(res.result, Some("  "));
        assert_eq!(res.state, "12");
    }
}