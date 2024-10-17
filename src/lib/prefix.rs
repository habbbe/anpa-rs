use core::borrow::Borrow;

/// Trait for a type that can remove a prefix in the collection `Parent`.
pub trait Prefix<Parent, Result>: Copy {
    /// Remove the prefix described by this prefix from the haystack if present, and if so,
    /// return it along with the rest of the haystack.
    fn remove_prefix(&self, haystack: Parent) -> Option<(Result, Parent)>;
}

impl<'a, T: PartialEq + Copy> Prefix<&'a [T], T> for T {
    fn remove_prefix(&self, haystack: &'a [T]) -> Option<(T, &'a [T])> {
        haystack.split_first()
            .filter(|x| x.0 == self)
            .map(|x| (*x.0, x.1))
    }
}

impl<'a, T: PartialEq + Copy, S: Borrow<[T]> + Copy> Prefix<&'a [T], &'a [T]> for S {
    fn remove_prefix(&self, haystack: &'a [T]) -> Option<(&'a [T], &'a [T])> {
        haystack.split_at_checked(self.borrow().len())
            .filter(|(prefix, _)| *prefix == self.borrow())
    }
}

impl<'a> Prefix<&'a str, char> for char {
    #[inline]
    fn remove_prefix(&self, haystack: &'a str) -> Option<(char, &'a str)> {
        haystack.strip_prefix(*self).map(|rest| (*self, rest))
    }
}

impl<'a, S: Borrow<str> + Copy> Prefix<&'a str, &'a str> for S {
    fn remove_prefix(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
        haystack.split_at_checked(self.borrow().len())
            .filter(|(prefix, _)| *prefix == self.borrow())
    }
}

/// Wrapper class to be used for prefix parsers that do not return any result.
#[derive(Clone, Copy)]
pub struct Ignore<T>(pub T);

impl<'a, T: PartialEq + Copy> Prefix<&'a [T], ()> for Ignore<T> {
    fn remove_prefix(&self, haystack: &'a [T]) -> Option<((), &'a [T])> {
        haystack.split_first()
            .filter(|x| x.0 == &self.0)
            .map(|x| ((), x.1))
    }
}

impl<'a, T: PartialEq + Copy> Prefix<&'a [T], ()> for Ignore<&[T]> {
    fn remove_prefix(&self, haystack: &'a [T]) -> Option<((), &'a [T])> {
        haystack.strip_prefix(self.0).map(|rest| ((), rest))
    }
}

impl<'a> Prefix<&'a str, ()> for Ignore<char> {
    fn remove_prefix(&self, haystack: &'a str) -> Option<((), &'a str)> {
        haystack.strip_prefix(self.0).map(|rest| ((), rest))
    }
}

impl<'a> Prefix<&'a str, ()> for Ignore<&str> {
    fn remove_prefix(&self, haystack: &'a str) -> Option<((), &'a str)> {
        haystack.strip_prefix(self.0).map(|rest| ((), rest))
     }
}