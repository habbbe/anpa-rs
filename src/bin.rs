use crate::Item::{Action, SyntaxError, Info, Separator};
use std::fs::File;
use std::io::{self, BufRead};
use std::time::{Instant};
use std::string::FromUtf8Error;
use anpa::{*};
use std::borrow::Borrow;

#[derive(Debug)]
enum Item {
    Action { name: String, com: String },
    Info { name: String, com: String },
    Separator,
    Space,
    SyntaxError { description: String }
}

fn action<I: Iterator<Item=impl Borrow<u8>>, I2: Iterator<Item=impl Borrow<u8>>>(name: I, com: I2) -> Item {
    Action {name: name.into_utf8_unchecked(), com: com.into_utf8_unchecked()}
}

fn info<I: Iterator<Item=impl Borrow<u8>>, I2: Iterator<Item=impl Borrow<u8>>>(name: I, com: I2) -> Item {
    Info {name: name.into_utf8_unchecked(), com: com.into_utf8_unchecked()}
}

fn syntax_error<I: Iterator<Item=impl Borrow<u8>>>(description: I) -> Item {
    SyntaxError {description: description.into_utf8_unchecked()}
}

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

    let rounds = 1;
    let file = File::open("hub").unwrap();
    let lines: Vec<String> = io::BufReader::new(file).lines().map(|l|l.expect("what")).collect();
    {
        use anpa::core::{parse, State};
        use anpa::parsers::{*};
        use anpa::combinators::{*};
        let parse_name = until_item(b'=');
        let parse_cmd = not_empty(rest());
        let parse_action = right(seq(b"Com:"), lift!(action, parse_name, parse_cmd));
        let parse_info = right(seq(b"Info:"), lift!(info, parse_name, parse_cmd));
        let parse_separator = lift!(|_| Item::Separator, seq(b"Separator"));
        let parse_space = lift!(|_| Item::Space, seq(b"Space"));
        let parse_error = lift!(syntax_error, rest());
        let item_parser = or!(parse_action, parse_info, parse_separator, parse_space, parse_error);
        let ignore = or_diff(item(b'#'), empty());
        let state_parser = or_diff(ignore, lift_to_state(|s: &mut Vec<_>, i| s.push(i), item_parser));

        let mut vec: Vec<Item> = Vec::with_capacity(100000);
        let now = Instant::now();

        for _ in 0..rounds {
            vec.clear();
            for l in &lines {
                let (_, r) = parse(state_parser, l.bytes(), &mut vec);
                if let None = r {
                    println!("No parse");
                    break
                }
            }
        }

        println!("N: {}, in {}ms", vec.len(), now.elapsed().as_millis());
    }

    {
        let mut vec: Vec<Item> = Vec::with_capacity(100000);
        let now = Instant::now();
        for _ in 0..rounds {
            vec.clear();
            for l in &lines {
                if let Some(i) = parse_handrolled(l) {
                    vec.push(i)
                } else {
                    println!("No parse");
                }
            }
        }

        println!("N: {}, in {}ms", vec.len(), now.elapsed().as_millis());
    }

    let v = vec![1,2,3];
    let v2 = vec![1,2,3];
    let it = v2.iter();

    println!("Are equal: {}", test(v, it));
}

fn test<T: PartialEq, X: Borrow<T>, I: Iterator<Item=X>>(v: Vec<T>, mut it: I) -> bool {
    for t in v.iter() {
        if let Some(n) = it.next() {
            if *n.borrow() != *t { return false }
        } else {
            return false;
        }
    }
    true
}

fn parse_handrolled(input: &str) -> Option<Item> {
    fn parse_command_tuple(input: &str) -> Option<(&str, &str)> {
        let equal_pos = input.find('=')?;
        if equal_pos == input.len() - 1 { return None }
        Some((&input[..equal_pos], &input[equal_pos..]))
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
        Some(Action {name: name.to_string(), com: com.to_string()})
    } else if let Some(rest) = parse_and_get_rest(input, "Info:") {
        let (name, com) = parse_command_tuple(rest)?;
        Some(Info {name: name.to_string(), com: com.to_string()})
    } else if let Some(_) = parse_and_get_rest(input, "Separator") {
        Some(Separator)
    } else if let Some(_) = parse_and_get_rest(input, "Space") {
        Some(Item::Space)
    } else {
        Some(SyntaxError {description: input.to_string()})
    }
}
