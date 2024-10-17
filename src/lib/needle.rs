use core::borrow::Borrow;

pub type NeedleLen = usize;

/// Trait for a type that can be sought after in the collection `Parent`.
pub trait Needle<Parent, Result>: Copy {
    /// Find the index of the needle in the provided haystack.
    fn find_in(&self, haystack: Parent) -> Option<(NeedleLen, usize)>;
}

impl<'a, T: PartialEq + Copy> Needle<&'a [T], T> for T {
    fn find_in(&self, haystack: &[T]) -> Option<(usize, usize)> {
        haystack.iter()
            .position(|x| x == self)
            .map(|pos| (1, pos))
    }
}

impl<'a, T: PartialEq + Copy, S: Borrow<[T]> + Copy> Needle<&'a [T], &'a [T]> for S {
    fn find_in(&self, haystack: &[T]) -> Option<(NeedleLen, usize)> {
        haystack.windows(self.borrow().len())
            .position(|w| w == self.borrow())
            .map(|pos| (self.borrow().len(), pos))
    }
}

impl<'a> Needle<&'a str, char> for char {
    #[inline]
    fn find_in(&self, haystack: &str) -> Option<(NeedleLen, usize)> {
        haystack.find(*self)
            .map(|pos| (self.len_utf8(), pos))
    }
}

impl<'a, S: Borrow<str> + Copy> Needle<&'a str, &'a str> for S {
    fn find_in(&self, haystack: &str) -> Option<(NeedleLen, usize)> {
        haystack.find(self.borrow())
            .map(|pos| (self.borrow().len(), pos))
    }
}