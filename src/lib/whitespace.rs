use crate::{charlike::CharLike, core::Parser, prefix::Prefix, slicelike::SliceLike};

/// Trait for inputs that can be trimmed, i.e. having ASCII whitespace removed
/// from the start.
pub trait TrimmableAscii: SliceLike {
    fn prefix() -> impl Prefix<Self, Self>;
}

/// Trait for inputs that can be trimmed, i.e. having UTF-8 whitespace removed
/// from the start.
pub trait TrimmableUtf8: SliceLike {
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

impl TrimmableUtf8 for &str {
    fn prefix() -> impl Prefix<Self, Self> {
        Utf8Whitespace()
    }
}

/// Create a parser that parses and returns ASCII whitespace.
#[inline]
pub const fn ascii_whitespace<I: TrimmableAscii, S>() -> impl Parser<I, I, S> {
    take!(I::prefix())
}

/// Create a parser that parses and ignores ASCII whitespace.
#[inline]
pub const fn skip_ascii_whitespace<I: TrimmableAscii, S>() -> impl Parser<I, (), S> {
    skip!(I::prefix())
}

/// Create a parser that parses and returns UTF-8 whitespace.
#[inline]
pub const fn whitespace<I: TrimmableUtf8, S>() -> impl Parser<I, I, S> {
    take!(I::prefix())
}

/// Create a parser that parses and ignores UTF-8 whitespace.
#[inline]
pub const fn skip_whitespace<I: TrimmableUtf8, S>() -> impl Parser<I, (), S> {
    skip!(I::prefix())
}

/// `Prefix` that matches zero or more ASCII whitespaces.
#[derive(Clone, Copy)]
pub struct AsciiWhitespace();

/// `Prefix` that matches zero or more UTF-8 whitespaces.
#[derive(Clone, Copy)]
pub struct Utf8Whitespace();

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

macro_rules! impl_whitespace_prefix_str {
    ($id:ident, $trim_fn:ident) => {
        impl<'a> Prefix<&'a str, &'a str> for $id {
            fn take_prefix(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
                let trimmed = haystack.$trim_fn();
                Some((&haystack[..haystack.len() - trimmed.len()], trimmed))
            }

            fn skip_prefix(&self, haystack: &'a str) -> Option<&'a str> {
                Some(haystack.$trim_fn())
            }
        }
    };
}

impl_whitespace_prefix_str!(AsciiWhitespace, trim_ascii_start);
impl_whitespace_prefix_str!(Utf8Whitespace, trim_start);

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