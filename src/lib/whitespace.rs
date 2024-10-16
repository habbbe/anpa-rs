use crate::{asciilike::AsciiLike, needle::{Needle, NeedleLen}};

/// Count ASCII whitespaces in a slice with ASCII like elements.
fn count_ascii_whitespace_slice<A: AsciiLike>(haystack: &[A]) -> usize {
    haystack.iter().position(|a| !a.is_whitespace_ascii()).unwrap_or(haystack.len())
}

/// Needle that consumes and ignores ASCII whitespaces from a slice with ASCII like elements.
#[derive(Clone, Copy)]
pub struct SliceWhitespaceIgnore();
impl<'a, A: AsciiLike + Copy> Needle<&'a [A], ()> for SliceWhitespaceIgnore {
    fn find_in(&self, haystack: &'a [A]) -> Option<(NeedleLen, usize)> {
        Some((count_ascii_whitespace_slice(haystack), 0))
    }

    fn remove_prefix(&self, haystack: &'a [A]) -> Option<((), &'a [A])> {
        Some(((), &haystack[count_ascii_whitespace_slice(haystack)..]))
    }
}

/// Needle that consumes and ignores ASCII whitespaces from a `&str``.
#[derive(Clone, Copy)]
pub struct StrWhitespaceIgnore();
impl<'a> Needle<&'a str, ()> for StrWhitespaceIgnore {
    #[inline]
    fn find_in(&self, haystack: &str) -> Option<(NeedleLen, usize)> {
        let len = haystack.len() - haystack.trim_ascii_start().len();
        Some((len, 0))
    }

    #[inline]
    fn remove_prefix(&self, haystack: &'a str) -> Option<((), &'a str)> {
        Some(((), haystack.trim_ascii_start()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{core::parse, whitespace::StrWhitespaceIgnore};

    use super::SliceWhitespaceIgnore;

    #[test]
    fn test_whitespace_consumer_u8() {
        let input = &[b' ', b' ', 1, 2];
        let res = parse(elem!(SliceWhitespaceIgnore()), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &[1, 2]);
    }

    #[test]
    fn test_whitespace_consumer_u8_nothing() {
        let input = &[1, 2];
        let res = parse(elem!(SliceWhitespaceIgnore()), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &[1, 2]);
    }

    #[test]
    fn test_whitespace_consumer_char() {
        let input = &[' ', ' ', '1', '2'];
        let res = parse(elem!(SliceWhitespaceIgnore()), input.as_slice());
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, &['1', '2']);
    }

    #[test]
    fn test_whitespace_consumer_str() {
        let input = "  12";
        let res = parse(elem!(StrWhitespaceIgnore()), input);
        assert_eq!(res.result, Some(()));
        assert_eq!(res.state, "12");
    }
}