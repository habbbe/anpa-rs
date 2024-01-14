use core::borrow::Borrow;
use std::{slice::Iter, str::Chars};

pub trait AsciiLike: SliceLike<Item = Self::T> {
    type T: Copy;
    const MINUS: Self::Item;
    const PERIOD: Self::Item;
    fn to_digit(item: Self::RefItem) -> Option<u8>;
}

impl AsciiLike for &str {
    type T = char;
    const MINUS: Self::Item = '-';
    const PERIOD: Self::Item = '.';

    #[inline(always)]
    fn to_digit(item: Self::RefItem) -> Option<u8> {
        item.to_digit(10).map(|c| c as u8)
    }
}

impl AsciiLike for &[u8] {
    type T = u8;
    const MINUS: Self::Item = b'-';
    const PERIOD: Self::Item = b'.';
    #[inline(always)]
    fn to_digit(item: Self::RefItem) -> Option<u8> {
        (*item >= b'0' && *item <= b'9').then_some(*item - b'0')
    }
}

pub trait SliceLike: Sized + Copy {
    type Item: PartialEq;
    type RefItem: PartialEq + Copy;
    type Iter: Iterator<Item = Self::RefItem>;
    fn slice_iter(self) -> Self::Iter;
    fn slice_first(self) -> Option<Self::RefItem>;
    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize>;
    fn slice_find_seq<S: Borrow<Self>>(self, item: S) -> Option<usize>;
    fn slice_find_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> Option<usize>;
    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool;
    fn slice_starts_with_seq(self, item: Self) -> bool;
    fn slice_len(self) -> usize;
    fn slice_from(self, from: usize) -> Self;
    fn slice_to(self, to: usize) -> Self;
    fn slice_from_to(self, from: usize, to: usize) -> Self;
    fn slice_split_at(self, at: usize) -> (Self, Self);
    fn slice_is_empty(&self) -> bool;
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

    fn slice_find_seq<S: Borrow<Self>>(self, item: S) -> Option<usize> {
        self.windows(item.borrow().len()).position(|w| &w == item.borrow())
    }

    fn slice_find_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> Option<usize> {
        self.iter().position(pred)
    }

    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool {
        self.first().filter(|x| *x == item.borrow()).is_some()
    }

    fn slice_starts_with_seq(self, item: Self) -> bool {
        self.starts_with(item)
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

    fn slice_find_seq<S: Borrow<Self>>(self, item: S) -> Option<usize> {
        self.find(item.borrow())
    }

    fn slice_find_pred(self, pred: impl Fn(Self::RefItem) -> bool) -> Option<usize> {
        self.find(pred)
    }

    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool {
        self.starts_with(*item.borrow())
    }

    fn slice_starts_with_seq(self, item: Self) -> bool {
        self.starts_with(item)
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
