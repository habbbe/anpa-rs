use core::{slice::Iter, str::Chars};

/// Share trait for "slicable" inputs. Anpa can be used to parse types implementing this trait.
pub trait SliceLike: Sized + Copy {
    type RefItem: PartialEq + Copy;
    type Iter: Iterator<Item = Self::RefItem>;

    /// Get an iterator for this input.
    fn slice_iter(self) -> Self::Iter;

    /// Get the first item of this input along with the rest, if it satisfies the provided predicate.
    fn slice_first_if(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<(Self::RefItem, Self)>;

    /// Get the (optional) index of the first item that matches `pred`.
    fn slice_find_pred(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<usize>;

    /// Get the current length of the input.
    fn slice_len(self) -> usize;

    /// Create a slice from index `from` until the end of the input.
    fn slice_from(self, from: usize) -> Self;

    /// Create a slice from the start of the input until `to` (exclusive).
    fn slice_to(self, to: usize) -> Self;

    /// Split the input at index `at`.
    fn slice_split_at(self, at: usize) -> (Self, Self);

    /// Check if the input is empty.
    fn slice_is_empty(&self) -> bool;
}

impl<'a, A: PartialEq> SliceLike for &'a [A] {
    type RefItem = &'a A;
    type Iter = Iter<'a, A>;

    fn slice_iter(self) -> Self::Iter {
        self.iter()
    }

    fn slice_first_if(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<(Self::RefItem, Self)> {
        self.split_first().filter(|x| pred(x.0))
    }

    fn slice_find_pred(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<usize> {
        self.iter().position(|x| pred(x))
    }

    fn slice_len(self) -> usize {
        self.len()
    }

    fn slice_from(self, from: usize) -> Self {
        &self[from..]
    }

    fn slice_to(self, to: usize) -> Self {
        &self[..to]
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at(at)
    }

    fn slice_is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'a> SliceLike for &'a str {
    type RefItem = char;
    type Iter = Chars<'a>;

    fn slice_iter(self) -> Self::Iter {
        self.chars()
    }

    fn slice_first_if(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<(Self::RefItem, Self)> {
        let mut chars = self.chars();
        let first = chars.next()?;
        pred(first).then_some((first, chars.as_str()))
    }

    fn slice_find_pred(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<usize> {
        self.find(|c| pred(c))
    }

    fn slice_len(self) -> usize {
        self.len()
    }

    fn slice_from(self, from: usize) -> Self {
        &self[from..]
    }

    fn slice_to(self, to: usize) -> Self {
        &self[..to]
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at(at)
    }

    fn slice_is_empty(&self) -> bool {
        self.is_empty()
    }
}
