use std::iter::{Take};
use crate::core::{*};

pub fn item<T: PartialEq + Copy, I: Iterator<Item=T> + Clone, S>(item: T) -> impl Parser<T, I, S, T> + Copy {
    create_parser!(s, {
        let pos = s.iterator.clone();
        match s.iterator.next() {
            Some(x) if x == item => Some(x),
            _                    => { s.iterator = pos; None }
        }
    })
}

pub fn until_item<T: PartialEq + Copy, I: Iterator<Item=T> + Clone, S>(item: T) -> impl Parser<T, I, S, Take<I>> + Copy {
    create_parser!(s, {
        let res = s.iterator.clone();
        let len = s.iterator.position(|i| i == item)?;
        Some(res.take(len))
    })
}

pub fn rest<T, I: Iterator<Item=T> + Clone, S>() -> impl Parser<T, I, S, I> + Copy {
    create_parser!(s, {
        let res = s.iterator.clone();
        s.iterator.by_ref().last();
        Some(res)
    })
}

pub fn seq<T: PartialEq, I: Iterator<Item=T> + Clone, S>(items: &[T]) -> impl Parser<T, I, S, &[T]> + Copy {
    create_parser!(s, {
        let orig = s.iterator.clone();
        for i in items {
            match s.iterator.next() {
                Some(x) if x == *i => {},
                _ => { s.iterator = orig; return None }
            }
        }
        Some(items)
    })
}

pub fn empty<T, I: Iterator<Item=T> + Clone, S>() -> impl Parser<T, I, S, ()> + Copy {
    create_parser!(s, {
        if s.iterator.clone().next().is_none() {
            Some(())
        } else {
            None
        }
    })
}