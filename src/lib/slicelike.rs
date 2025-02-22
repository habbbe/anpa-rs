use core::{ops::{Add, AddAssign, Sub, SubAssign}, slice::Iter, str::Chars};

/// Share trait for "slicable" inputs. Anpa can be used to parse types implementing this trait.
pub trait SliceLike: Copy {
    type Idx: Add<Output = Self::Idx> + AddAssign + Sub<Output = Self::Idx> +
                           SubAssign + PartialEq + PartialOrd + From<bool> + Default + Copy;
    type RefItem: Copy;
    type Iter: Iterator<Item = Self::RefItem>;

    /// Get an index from `usize`.
    fn slice_idx_from_offset(self, idx: usize) -> Self::Idx;

    /// Get an iterator for this input.
    fn slice_iter(self) -> Self::Iter;

    /// Get the first item of this input along with the rest, if it satisfies the provided predicate.
    fn slice_first_if(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<(Self::RefItem, Self)>;

    /// Get the (optional) index of the first item that matches `pred`.
    fn slice_find_pred(self, pred: impl FnMut(Self::RefItem) -> bool + Copy) -> Option<Self::Idx>;

    /// Get the current length of the input.
    fn slice_len(self) -> Self::Idx;

    /// Create a slice from index `from` until the end of the input.
    fn slice_from(self, from: Self::Idx) -> Self;

    /// Create a slice from the start of the input until `to` (exclusive).
    fn slice_to(self, to: Self::Idx) -> Self;

    /// Split the input at index `at`.
    fn slice_split_at(self, at: Self::Idx) -> (Self, Self);

    /// Check if the input is empty.
    fn slice_is_empty(&self) -> bool;
}

impl<'a, A> SliceLike for &'a [A] {
    type Idx = usize;
    type RefItem = &'a A;
    type Iter = Iter<'a, A>;

    #[inline(always)]
    fn slice_idx_from_offset(self, idx: usize) -> Self::Idx {
        idx
    }

    fn slice_iter(self) -> Self::Iter {
        self.iter()
    }

    fn slice_first_if(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<(Self::RefItem, Self)> {
        self.split_first().filter(|x| pred(x.0))
    }

    fn slice_find_pred(self, pred: impl FnMut(Self::RefItem) -> bool + Copy) -> Option<usize> {
        self.iter().position(pred)
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
    type Idx = usize;
    type RefItem = char;
    type Iter = Chars<'a>;

    #[inline(always)]
    fn slice_idx_from_offset(self, idx: usize) -> Self::Idx {
        idx
    }

    fn slice_iter(self) -> Self::Iter {
        self.chars()
    }

    fn slice_first_if(self, pred: impl FnOnce(Self::RefItem) -> bool + Copy) -> Option<(Self::RefItem, Self)> {
        let mut chars = self.chars();
        let first = chars.next()?;
        pred(first).then_some((first, chars.as_str()))
    }

    fn slice_find_pred(self, pred: impl FnMut(Self::RefItem) -> bool + Copy) -> Option<usize> {
        self.find(pred)
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

/// A trait for types that can be converted to `&[u8]`.
pub trait ContiguousBytes {
    fn to_u8_slice(&self) -> &[u8];
}

impl ContiguousBytes for &str {
    #[inline(always)]
    fn to_u8_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ContiguousBytes for &[u8] {
    #[inline(always)]
    fn to_u8_slice(&self) -> &[u8] {
        self
    }
}