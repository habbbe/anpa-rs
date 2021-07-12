use std::iter::{Take};
use crate::core::{*};
use std::borrow::Borrow;

pub fn item<T: PartialEq, X: Borrow<T>, Y: Borrow<T> + Copy, I: Iterator<Item=X> + Clone, S>(item: Y) -> impl Parser<T, X, I, S, X> + Copy {
    create_parser!(s, {
        let pos = s.iterator.clone();
        match s.iterator.next() {
            Some(x) if x.borrow() == item.borrow() => Some(x),
            _                    => { s.iterator = pos; None }
        }
    })
}

pub fn until_item<T: PartialEq, X: Borrow<T>, Y: Borrow<T> + Copy, I: Iterator<Item=X> + Clone, S>(item: Y) -> impl Parser<T, X, I, S, Take<I>> + Copy {
    create_parser!(s, {
        let res = s.iterator.clone();
        let len = s.iterator.position(|i| i.borrow() == item.borrow())?;
        Some(res.take(len))
    })
}

pub fn rest<T, X: Borrow<T>, I: Iterator<Item=X> + Clone, S>() -> impl Parser<T, X, I, S, I> + Copy {
    create_parser!(s, {
        let res = s.iterator.clone();
        s.iterator.by_ref().last();
        Some(res)
    })
}

pub fn seq<T: PartialEq, X: Borrow<T>, I: Iterator<Item=X> + Clone, S>(items: &[T]) -> impl Parser<T, X, I, S, &[T]> + Copy {
    create_parser!(s, {
        let orig = s.iterator.clone();
        for i in items {
            match s.iterator.next() {
                Some(x) if *x.borrow() == *i => {},
                _ => { s.iterator = orig; return None }
            }
        }
        Some(items)
    })
}

pub fn empty<T, X: Borrow<T>, I: Iterator<Item=X> + Clone, S>() -> impl Parser<T, X, I, S, ()> + Copy {
    create_parser!(s, {
        if s.iterator.clone().next().is_none() {
            Some(())
        } else {
            None
        }
    })
}