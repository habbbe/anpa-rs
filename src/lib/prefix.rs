use core::borrow::Borrow;

/// Trait for a type that can remove a prefix in the collection `Parent`.
pub trait Prefix<Parent, Result>: Copy {
    /// Remove the prefix described by this prefix from the haystack if present, and if so,
    /// return it along with the rest of the haystack.
    fn pop_prefix(&self, haystack: Parent) -> Option<(Result, Parent)>;

    /// Remove the prefix described by this prefix from the haystack if present, and return
    /// the rest of the haystack (result is ignored).
    fn drop_prefix(&self, haystack: Parent) -> Option<Parent> {
        Some(self.pop_prefix(haystack)?.1)
    }
}

impl<'a, T: PartialEq + Copy> Prefix<&'a [T], T> for T {
    fn pop_prefix(&self, haystack: &'a [T]) -> Option<(T, &'a [T])> {
        haystack.split_first()
            .filter(|x| x.0 == self)
            .map(|x| (*x.0, x.1))
    }
}

impl<'a, T: PartialEq + Copy, S: Borrow<[T]> + Copy> Prefix<&'a [T], &'a [T]> for S {
    fn pop_prefix(&self, haystack: &'a [T]) -> Option<(&'a [T], &'a [T])> {
        haystack.split_at_checked(self.borrow().len())
            .filter(|(prefix, _)| *prefix == self.borrow())
    }

    fn drop_prefix(&self, haystack: &'a [T]) -> Option<&'a [T]> {
        haystack.strip_prefix(self.borrow())
    }
}

impl<'a> Prefix<&'a str, char> for char {
    fn pop_prefix(&self, haystack: &'a str) -> Option<(char, &'a str)> {
        let rest = haystack.strip_prefix(*self)?;
        Some((*self, rest))
    }

    fn drop_prefix(&self, haystack: &'a str) -> Option<&'a str> {
        haystack.strip_prefix(*self)
    }
}

impl<'a, S: Borrow<str> + Copy> Prefix<&'a str, &'a str> for S {
    fn pop_prefix(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
        haystack.split_at_checked(self.borrow().len())
            .filter(|(prefix, _)| *prefix == self.borrow())
    }

    fn drop_prefix(&self, haystack: &'a str) -> Option<&'a str> {
        haystack.strip_prefix(self.borrow())
    }
}