use core::borrow::Borrow;
use std::{slice::Iter, str::Chars};

/// Share trait for "slicable" inputs. Anpa can be used to parse types implementing this trait.
pub trait SliceLike: Sized + Copy {
    type Item: PartialEq;
    type RefItem: PartialEq + Copy;
    type Iter: Iterator<Item = Self::RefItem>;

    /// Get an iterator for this input.
    fn slice_iter(self) -> Self::Iter;

    /// Get the (optional) first item of this input.
    fn slice_first(self) -> Option<Self::RefItem>;

    /// Get the (optional) index of the requested `item`.
    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize>;

    /// Get the (optional) index of the requested `seq`uence.
    fn slice_find_seq<S: Borrow<Self>>(self, seq: S) -> Option<usize>;

    /// Get the (optional) index of the first item that matches `pred`.
    fn slice_find_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> Option<usize>;

    /// Return whether the input starts with `item`.
    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool;

    /// Return whether the input starts with predicate `p`.
    fn slice_starts_with_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> bool;

    /// Return whether the input starts with `seq`.
    fn slice_starts_with_seq(self, seq: Self) -> bool;

    /// Get the current length of the input.
    fn slice_len(self) -> usize;

    /// Create a slice from index `from` until the end of the input.
    fn slice_from(self, from: usize) -> Self;

    /// Create a slice from the start of the input until `to` (exclusive).
    fn slice_to(self, to: usize) -> Self;

    /// Create a slice from index `from` until `to` (exclusive).
    fn slice_from_to(self, from: usize, to: usize) -> Self;

    /// Split the input at index `at`.
    fn slice_split_at(self, at: usize) -> (Self, Self);

    /// Check if the input is empty.
    fn slice_is_empty(&self) -> bool;

    /// Check if a reference of type `&Self::Item` is equal to a `Self::RefItem`.
    fn slice_item_eq_ref_item(a: &Self::Item, b: Self::RefItem) -> bool;
}

impl<'a, A: PartialEq> SliceLike for &'a [A] {
    type Item = A;
    type RefItem = &'a A;
    type Iter = Iter<'a, A>;

    fn slice_iter(self) -> Self::Iter {
        self.iter()
    }

    fn slice_first(self) -> Option<Self::RefItem> {
        self.first()
    }

    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize> {
        self.iter().position(|x| x == item.borrow())
    }

    fn slice_find_seq<S: Borrow<Self>>(self, seq: S) -> Option<usize> {
        self.windows(seq.borrow().len()).position(|w| &w == seq.borrow())
    }

    fn slice_find_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> Option<usize> {
        self.iter().position(pred)
    }

    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool {
        self.first().filter(|x| *x == item.borrow()).is_some()
    }

    fn slice_starts_with_seq(self, seq: Self) -> bool {
        self.starts_with(seq)
    }

    fn slice_starts_with_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> bool {
        self.first().filter(|c| pred(*c)).is_some()
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

    fn slice_from_to(self, from: usize, to: usize) -> Self {
        &self[from..to]
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at(at)
    }

    fn slice_is_empty(&self) -> bool {
        self.is_empty()
    }

    fn slice_item_eq_ref_item(a: &Self::Item, b: Self::RefItem) -> bool {
        a == b
    }
}

impl<'a> SliceLike for &'a str {
    type Item = char;
    type RefItem = char;
    type Iter = Chars<'a>;

    fn slice_iter(self) -> Self::Iter {
        self.chars()
    }

    fn slice_first(self) -> Option<Self::RefItem> {
        self.chars().next()
    }

    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize> {
        self.find(*item.borrow())
    }

    fn slice_find_seq<S: Borrow<Self>>(self, seq: S) -> Option<usize> {
        self.find(seq.borrow())
    }

    fn slice_find_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> Option<usize> {
        self.find(pred)
    }

    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool {
        self.starts_with(*item.borrow())
    }

    fn slice_starts_with_seq(self, seq: Self) -> bool {
        self.starts_with(seq)
    }

    fn slice_starts_with_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> bool {
        self.starts_with(pred)
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

    fn slice_from_to(self, from: usize, to: usize) -> Self {
        &self[from..to]
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at(at)
    }

    fn slice_is_empty(&self) -> bool {
        self.is_empty()
    }

    fn slice_item_eq_ref_item(a: &Self::Item, b: Self::RefItem) -> bool {
        a == &b
    }
}
