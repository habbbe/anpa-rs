use core::borrow::Borrow;

pub type NeedleLen = usize;

/// Trait for a type that can be sought after in the collection `Parent`.
pub trait Needle<Parent, Result>: Copy {

    /// Find the index of the needle in the provided haystack.
    fn find_in(&self, haystack: Parent) -> Option<(NeedleLen, usize)>;

    /// Remove the prefix described by this needle from the haystack if present, and if so,
    /// return it along with the rest of the haystack.
    fn remove_prefix(&self, haystack: Parent) -> Option<(Result, Parent)>;
}

impl<'a, T: PartialEq + Copy> Needle<&'a [T], T> for T {
    fn find_in(&self, haystack: &[T]) -> Option<(usize, usize)> {
        haystack.iter()
            .position(|x| x == self)
            .map(|pos| (1, pos))
    }

    fn remove_prefix(&self, haystack: &'a [T]) -> Option<(T, &'a [T])> {
        haystack.split_first()
            .filter(|x| x.0 == self)
            .map(|x| (*x.0, x.1))
    }
}

impl<'a, T: PartialEq + Copy, S: Borrow<[T]> + Copy> Needle<&'a [T], &'a [T]> for S {
    fn find_in(&self, haystack: &[T]) -> Option<(NeedleLen, usize)> {
        haystack.windows(self.borrow().len())
            .position(|w| w == self.borrow())
            .map(|pos| (self.borrow().len(), pos))
    }

    fn remove_prefix(&self, haystack: &'a [T]) -> Option<(&'a [T], &'a [T])> {
        haystack.split_at_checked(self.borrow().len())
            .filter(|(prefix, _)| *prefix == self.borrow())
    }
}

impl<'a> Needle<&'a str, char> for char {
    #[inline]
    fn find_in(&self, haystack: &str) -> Option<(NeedleLen, usize)> {
        haystack.find(*self)
            .map(|pos| (self.len_utf8(), pos))
    }

    #[inline]
    fn remove_prefix(&self, haystack: &'a str) -> Option<(char, &'a str)> {
        haystack.strip_prefix(*self).map(|rest| (*self, rest))
    }
}

impl<'a, S: Borrow<str> + Copy> Needle<&'a str, &'a str> for S {

    fn find_in(&self, haystack: &str) -> Option<(NeedleLen, usize)> {
        haystack.find(self.borrow())
            .map(|pos| (self.borrow().len(), pos))
    }

    fn remove_prefix(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
        haystack.split_at_checked(self.borrow().len())
            .filter(|(prefix, _)| *prefix == self.borrow())
    }
}