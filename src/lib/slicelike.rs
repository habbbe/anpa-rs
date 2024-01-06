use core::borrow::Borrow;

pub trait SliceLike: Sized + Copy {
    type Item: PartialEq;
    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize>;
    fn slice_find_seq<S: Borrow<Self>>(self, item: S) -> Option<usize>;
    fn slice_starts_with<I: Borrow<Self::Item>>(self, item: I) -> bool;
    fn slice_starts_with_seq(self, item: Self) -> bool;
    fn slice_len(self) -> usize;
    fn slice_from(self, from: usize) -> Self;
    fn slice_to(self, to: usize) -> Self;
    fn slice_from_to(self, from: usize, to: usize) -> Self;
    fn slice_split_at(self, at: usize) -> (Self, Self);
    fn slice_is_empty(&self) -> bool;
}

impl<A: PartialEq> SliceLike for &[A] {
    type Item = A;

    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize> {
        self.iter().position(|x| x == item.borrow())
    }

    fn slice_find_seq<S: Borrow<Self>>(self, item: S) -> Option<usize> {
        self.windows(item.borrow().len()).position(|w| &w == item.borrow())
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
}

impl SliceLike for &str {
    type Item = char;

    fn slice_find<I: Borrow<Self::Item> + Copy>(self, item: I) -> Option<usize> {
        self.find(*item.borrow())
    }

    fn slice_find_seq<S: Borrow<Self>>(self, item: S) -> Option<usize> {
        self.find(item.borrow())
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
}
