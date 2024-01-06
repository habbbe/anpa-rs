use crate::Item::{Action, SyntaxError, Info, Separator};
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Index;
use std::time::{Instant};
use std::string::FromUtf8Error;
use std::borrow::{Borrow, BorrowMut};
use anpa::core::Nothing;

#[derive(Debug)]
enum Item<'a> {
    Action { name: &'a str, com: &'a str },
    Info { name: &'a str, com: &'a str },
    Separator,
    Space,
    Ignore,
    SyntaxError { description: &'a str }
}

// fn action<I: Iterator<Item=impl Borrow<u8>>, I2: Iterator<Item=impl Borrow<u8>>>(name: I, com: I2) -> Item {
//     Action {name: name.into_utf8_unchecked(), com: com.into_utf8_unchecked()}
// }

// fn info<I: Iterator<Item=impl Borrow<u8>>, I2: Iterator<Item=impl Borrow<u8>>>(name: I, com: I2) -> Item {
//     Info {name: name.into_utf8_unchecked(), com: com.into_utf8_unchecked()}
// }

// fn syntax_error<I: Iterator<Item=impl Borrow<u8>>>(description: I) -> Item {
//     SyntaxError {description: description.into_utf8_unchecked()}
// }

trait ToUtf8String {
    fn into_utf8_string(self) -> Result<String, FromUtf8Error>;
    fn into_utf8_unchecked(self) -> String;
}


impl<I: Iterator<Item=impl Borrow<u8>>> ToUtf8String for I {
    fn into_utf8_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.map(|x|*x.borrow()).collect())
    }

    fn into_utf8_unchecked(self) -> String {
        unsafe {
            String::from_utf8_unchecked(self.map(|x|*x.borrow()).collect())
        }
    }
}

fn main() {

    let file = File::open("/home/habbbe/.hubbbench").unwrap();
    let lines: Vec<String> = io::BufReader::new(file).lines().map(Result::unwrap).collect();

    {
        // let mut vec: Vec<Item> = Vec::with_capacity(100000);
        // let now = Instant::now();
        // for _ in 0..rounds {
        //     vec.clear();
        //     for l in &lines {
        //         if let Some(i) = parse_handrolled(l) {
        //             vec.push(i)
        //         } else {
        //             println!("No parse");
        //         }
        //     }
        // }
        //
        // println!("N: {}, in {}ms", vec.len(), now.elapsed().as_millis());
    }
    {
        // use anpa::{*};
        // use anpa::core::{parse, State};
        // use anpa::parsers::{*};
        // use anpa::combinators::{*};
        // let parse_name = until_item(b'=');
        // let parse_cmd = not_empty(rest());
        // let parse_action = right(seq(b"Com:"), lift!(action, parse_name, parse_cmd));
        // let parse_info = right(seq(b"Info:"), lift!(info, parse_name, parse_cmd));
        // let parse_separator = lift!(|_| Item::Separator, seq(b"Separator"));
        // let parse_space = lift!(|_| Item::Space, seq(b"Space"));
        // let parse_error = lift!(syntax_error, rest());
        // let item_parser = or!(parse_action, parse_info, parse_separator, parse_space, parse_error);
        // let ignore = or_diff!(item(b'#'), empty());
        // let state_parser = or_diff!(ignore, lift_to_state(|s: &mut Vec<_>, i| s.push(i), item_parser));
        //
        // let mut vec: Vec<Item> = Vec::with_capacity(100000);
        // let now = Instant::now();
        //
        // for _ in 0..rounds {
        //     vec.clear();
        //     for l in &lines {
        //         let (_, r) = parse(state_parser, l.bytes(), &mut vec);
        //         if let None = r {
        //             println!("No parse");
        //             break
        //         }
        //     }
        // }
        //
        // println!("N: {}, in {}ms", vec.len(), now.elapsed().as_millis());
    }

    {
        use anpa::{*};

        let parse_name = until_seq("=");
        let parse_cmd = not_empty(rest());
        let parse_action = right(seq("Com:"), lift2!(action2, parse_name, parse_cmd));
        let parse_info = right(seq("Info:"), lift2!(info2, parse_name, parse_cmd));
        let parse_separator = lift2!(|_| Item::Separator, seq("Separator"));
        let parse_space = lift2!(|_| Item::Space, seq("Space"));
        let parse_error = lift2!(syntax_error2, rest());
        let ignore = lift2!(|_| Item::Ignore, or_diff!(seq("#"), empty()));
        let item_parser = or!(parse_action, parse_info, parse_separator, parse_space, ignore, parse_error);

        let mut vec: Vec<Item> = Vec::with_capacity(1000000);
        let now = Instant::now();
        for l in &lines {
            // let r = parse_handrolled(l);
            // match r {

            let r = parse(item_parser, &l, ());
            match r.1 {
                None => {
                    println!("No parse");
                    break
                }
                Some(Item::Ignore) => {},
                Some(res) => vec.push(res)
            }
        }

        //
        println!("N: {}, in {}ms", vec.len(), now.elapsed().as_micros() as f64 / 1000.0);

        fn in_parens<'a, B>(thing: &'a str) -> impl Parser<&str, &str, B> {
            move |s| {
                or(seq(thing), right(seq("("), left(in_parens(thing), seq(")"))))(s)
            }
        }

        let p = in_parens("hej");

        let x = "(((((((((hej)))))))))";
        let num = parse(p, x, Nothing);
        if let Some(res) = num.1 {
            println!("Got the number!")
        }
    }
}

fn action2<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Action {name, com: com}
}

fn info2<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Info { name, com }
}

fn syntax_error2<'a>(description: &'a str) -> Item<'a> {
    SyntaxError {description}
}

pub trait Parser<I, O, S>: FnOnce(&mut ParserState<I, S>) -> Option<O> + Copy {}
impl<I, O, S, F: FnOnce(&mut ParserState<I, S>) -> Option<O> + Copy> Parser<I, O, S> for F {}

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

pub struct ParserState<T, B> {
    pub input: T,
    pub user_state: B,
}

macro_rules! starts_with {
    ($start:expr, $len:expr, $f:expr) => {
        move |s| {
            if $f(s.input, $start) {
                let res;
                (res, s.input) = s.input.slice_split_at($len);
                Some(res)
            } else {
                None
            }
        }
    }
}

fn item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    starts_with!(item, 1, SliceLike::slice_starts_with)
}

fn seq<I: SliceLike, B: Borrow<I> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    starts_with!(*item.borrow(), item.borrow().slice_len(), SliceLike::slice_starts_with_seq)
}

macro_rules! until_internal {
    ($item:expr, $len:expr, $f:expr) => {
        move |s| {
            let index = $f(s.input, $item)?;
            let res = s.input.slice_to(index);
            s.input = s.input.slice_from(index + $len);
            Some(res)
        }
    }
}

fn until_item<I: SliceLike, B: Borrow<I::Item> + Copy, S>(item: B) -> impl Parser<I, I, S> {
    until_internal!(item.borrow(), 1, SliceLike::slice_find)
}

fn until_seq<I: SliceLike, B: Borrow<I> + Copy, S>(seq: B) -> impl Parser<I, I, S> {
    until_internal!(*seq.borrow(), seq.borrow().slice_len(), SliceLike::slice_find_seq)
}

fn rest<I: SliceLike, S>() -> impl Parser<I, I, S> {
    move |s| {
        let all;
        (all, s.input) = s.input.slice_split_at(s.input.slice_len());
        Some(all)
    }
}

fn not_empty<I: SliceLike, O: SliceLike, S>(p: impl Parser<I, O, S>)
                               -> impl Parser<I, O, S> {
    move |s| {
        p(s).filter(|x| !x.slice_is_empty())
    }
}

fn empty<I: SliceLike, S>() -> impl Parser<I, (), S> {
    move |s| {
        if s.input.slice_is_empty() { Some(()) } else { None }
    }
}

fn success<I: SliceLike, S>() -> impl Parser<I, (), S> {
    move |_| {
        Some(())
    }
}

fn  right<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                   p2: impl Parser<I, O2, S>)
                                   ->  impl Parser<I, O2, S> {
    move |s| {
        p1(s)?;
        p2(s)
    }
}

fn  left<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                  p2: impl Parser<I, O2, S>)
                                   -> impl Parser<I, O1, S> {
    move |s| {
        if let a@Some(_) = p1(s) {
            p2(s)?;
            a
        } else {
            None
        }
    }
}

fn or<I: SliceLike, O, S>(p1: impl Parser<I, O, S>,
                          p2: impl Parser<I, O, S>)
                           -> impl Parser<I, O, S> {
    move |s| {
        let pos = s.input;
        if let a@Some(_) = p1(s) {
            a
        } else {
            s.input = pos;
            p2(s)
        }
    }
}

fn or_diff<I: SliceLike, S, O1, O2>(p1: impl Parser<I, O1, S>,
                                    p2: impl Parser<I, O2, S>)
                                     -> impl Parser<I, (), S> {
    move |s| {
        let pos = s.input;
        if p1(s).is_some() {
            Some(())
        } else {
            s.input = pos;
            p2(s)?;
            Some(())
        }
    }
}

fn lift_to_state<I: SliceLike, S, O1, O2>(f: impl FnOnce(&mut S, O1) -> O2 + Copy,
                                          p: impl Parser<I, O1, S>)
                                          -> impl Parser<I, O2, S> {
    move |s| {
        let res = p(s)?;
        Some(f(&mut s.user_state, res))
    }
}

pub fn parse<I: SliceLike, O, S>(p: impl Parser<I, O, S>,
                                         input: I,
                                         user_state: S) -> (ParserState<I, S>, Option<O>) {
    let mut parser_state = ParserState { input, user_state };
    let result = p(&mut parser_state);
    (parser_state, result)
}

fn parse_handrolled(input: &str) -> Option<Item> {
    fn parse_command_tuple(input: &str) -> Option<(&str, &str)> {
        let equal_pos = input.find("=")?;
        if equal_pos == input.len() - 1 { return None }
        Some((&input[..equal_pos], &input[(equal_pos + 1)..]))
    }

    fn parse_and_get_rest<'a>(source: &'a str, sought: &str) -> Option<&'a str> {
        if source.starts_with(sought) {
            Some(&source[sought.len()..])
        } else {
            None
        }
    }
    if let Some(rest) = parse_and_get_rest(input, "Com:") {
        let (name, com) = parse_command_tuple(rest)?;
        Some(Action {name, com})
    } else if let Some(rest) = parse_and_get_rest(input, "Info:") {
        let (name, com) = parse_command_tuple(rest)?;
        Some(Info {name, com})
    } else if parse_and_get_rest(input, "Separator").is_some() {
        Some(Separator)
    } else if parse_and_get_rest(input, "Space").is_some() {
        Some(Item::Space)
    } else if parse_and_get_rest(input, "#").is_some() || input.is_empty() {
        Some(Item::Ignore)
    } else {
        Some(SyntaxError {description: input})
    }
}
