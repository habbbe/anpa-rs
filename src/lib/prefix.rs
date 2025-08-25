use core::borrow::Borrow;

use crate::charlike::CharLike;

/// Trait for a type that can remove a prefix in the collection `Parent`.
pub trait Prefix<Parent, Result>: Copy {
    /// Remove the prefix described by this prefix from the haystack if present, and if so,
    /// return it along with the rest of the haystack.
    fn take_prefix(&self, haystack: Parent) -> Option<(Result, Parent)>;

    /// Remove the prefix described by this prefix from the haystack if present, and return
    /// the rest of the haystack (result is ignored).
    #[inline(always)]
    fn skip_prefix(&self, haystack: Parent) -> Option<Parent> {
        self.take_prefix(haystack).map(|x| x.1)
    }
}

impl<'a, T: PartialEq + Copy> Prefix<&'a [T], T> for T {
    fn take_prefix(&self, haystack: &'a [T]) -> Option<(T, &'a [T])> {
        haystack.split_first()
            .filter(|x| x.0 == self)
            .map(|x| (*self, x.1))
    }
}

impl<'a, T: PartialEq + Copy, S: Borrow<[T]> + Copy> Prefix<&'a [T], &'a [T]> for S {
    fn take_prefix(&self, haystack: &'a [T]) -> Option<(&'a [T], &'a [T])> {
        let needle = self.borrow();
        haystack.strip_prefix(needle)
            // SAFETY: Prefix guarantees valid bounds
            .map(|rest| unsafe { (haystack.get_unchecked(..needle.len()), rest) })
    }

    #[inline(always)]
    fn skip_prefix(&self, haystack: &'a [T]) -> Option<&'a [T]> {
        haystack.strip_prefix(self.borrow())
    }
}

impl<'a, C: CharLike> Prefix<&'a str, char> for C {
    fn take_prefix(&self, haystack: &'a str) -> Option<(char, &'a str)> {
        let c = self.as_char();
        haystack.strip_prefix(c)
            .map(|rest| (c, rest))
    }
}

impl<'a, S: Borrow<str> + Copy> Prefix<&'a str, &'a str> for S {
    fn take_prefix(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
        let needle = self.borrow();
        haystack.strip_prefix(needle)
            // SAFETY: Prefix guarantees valid bounds
            .map(|rest| unsafe { (haystack.get_unchecked(..needle.len()), rest) })
    }

    #[inline(always)]
    fn skip_prefix(&self, haystack: &'a str) -> Option<&'a str> {
        haystack.strip_prefix(self.borrow())
    }
}