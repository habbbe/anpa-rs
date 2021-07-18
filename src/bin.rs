use crate::Item::{Action, SyntaxError, Info, Separator};
use std::fs::File;
use std::io::{self, BufRead};
use std::time::{Instant};
use std::string::FromUtf8Error;
use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;

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
        let parse_name = until_item("=");
        let parse_cmd = not_empty(rest());
        // let parse_action = not_empty(rest());
        // let parse_action = right(rest(), rest());
        // let parse_action = right(rest(), rest());
        // let parse_action = not_empty(rest());
        // let parse_action2 = right(until_item("="), until_item("="));
        // let yo = right(parse_name, parse_name2);
        let parse_action = right(item("Com:"), lift2!(action2, parse_name, parse_cmd));
        let parse_info = right(item("Info:"), lift2!(info2, parse_name, parse_cmd));
        let parse_separator = lift2!(|_| Item::Separator, item("Separator"));
        let parse_space = lift2!(|_| Item::Space, item("Space"));
        let parse_error = lift2!(syntax_error2, rest());
        let item_parser = or!(parse_action, parse_info, parse_separator, parse_space, parse_error);
        let ignore = or_diff!(item("#"), empty());
        let state_parser = or_diff!(ignore, lift_to_state(|s: &mut Vec<_>, i| s.push(i), item_parser));
        // vec.clear();
        // parse(parse_action, "hej", &mut vec);
        // parse(parse_action, "hej", &mut vec);


        // let res = parse(parse_action, "hej", vec);
        // let res = parse(parse_action, "hej", res.0.user_state);
        // let mut state = Yo {input: "hej", user_state: vec};
        // let res = parse_action(&mut state);
        // let mut state = Yo {input: "hej", user_state: res.};
        // let res = parse_action(&mut state);
        // println!("{:?}, rest: {:?}", res.unwrap(), state.input);

        // let mut state = Yo {input: "hej", user_state: &mut vec};
        // parse_action(&mut state);
        // let mut state = Yo {input: "hej", user_state: &mut &vec};
        // parse_action(&mut state);
        //
        // let mut vec: Vec<Item> = Vec::with_capacity(100000);
        let now = Instant::now();
        let mut vec: Vec<Item> = Vec::with_capacity(100000);
        for _ in 0..rounds {
            // vec.clear();
            for l in &lines {
                let r = parse(state_parser, &l, vec);
                vec = r.0.user_state;
                // let (_, r) = parse(state_parser, &l, &mut vec);
                if let None = r.1 {
                    println!("No parse");
                    break
                }
            }
        }
        //
        println!("N: {}, in {}ms", vec.len(), now.elapsed().as_millis());
    }
}

fn action2(name: &str, com: &str) -> Item {
    Action {name: name.to_string(), com: com.to_string()}
}

fn info2(name: &str, com: &str) -> Item {
    Info {name: name.to_string(), com: com.to_string()}
}

fn syntax_error2(description: &str) -> Item {
    SyntaxError {description: description.to_string()}
}

pub struct Yo<'a, S, B> where B: BorrowMut<S>{
    pub input: &'a str,
    pub user_state: B,
    phantom: PhantomData<S>
}

fn item<'a, S, B>(item: &'a str) -> impl for <'b, 'c>FnOnce(&mut Yo<'b, S, B>) -> Option<&'b str> + 'a + Copy {
    move |s| {
        if s.input.starts_with(item) {
            let res = &s.input[..1];
            s.input = &s.input[1..];
            Some(res)
        } else {
            None
        }
    }
}

fn until_item<'a, S, B>(item: &'a str) -> impl for<'b>FnOnce(&mut Yo<'b, S, B>) -> Option<&'b str> + 'a + Copy {
    move |s| {
        let index = s.input.find(item)?;
        let res = &s.input[..index];
        s.input = &s.input[index + item.len()..];
        Some(res)
    }
}

fn rest<S, B>() -> impl for<'a>FnOnce(&mut Yo<'a, S, B>) -> Option<&'a str> + Copy {
    move |s| {
        let res = &s.input.clone();
        s.input = &s.input[s.input.len()..];
        Some(res)
    }
}
//
fn not_empty<S, B: BorrowMut<S>>(p: impl for<'a>FnOnce(&mut Yo<'a, S, B>) -> Option<&'a str> + Copy) -> impl for<'a>FnOnce(&mut Yo<'a, S, B>) -> Option<&'a str> + Copy {
    move |s| {
        p(s).filter(|x| !x.is_empty())
    }
}
//
fn empty<S, B>() -> impl FnOnce(&mut Yo<S, B>) -> Option<()> + Copy {
    move |s| {
        if s.input.is_empty() { Some(()) } else { None }
    }
}

fn success<S, B>() -> impl FnOnce(&mut Yo<S, B>) -> Option<()> + Copy {
    move |s| {
        Some(())
    }
}

fn  right<'a, S, B: BorrowMut<S>, R1, R2>(p1: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R1> + Copy, p2: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R2> + Copy) -> impl FnOnce(&mut Yo<'a, S, B>) -> Option<R2> + Copy {
    move |s: &mut _| {
        p1(s)?;
        p2(s)
    }
}
// fn right<'a, S, R1, R2>(p1: impl FnOnce(&mut Yo<'a, S>) -> Option<R1> + Copy, p2: impl FnOnce(&mut Yo<'a, S>) -> Option<R2> + Copy) -> impl FnOnce(&mut Yo<'a, S>) -> Option<R2> + Copy {
//     move |s| {
//         p1(s)?;
//         p2(s)
//     }
// }
// fn right<'a, S, R1, R2>(p1: impl FnOnce(&mut Yo<'a, S>) -> Option<R1> + Copy, p2: impl FnOnce(&mut Yo<'a, S>) -> Option<R2> + Copy) -> impl FnOnce(&mut Yo<'a, S>) -> Option<R2> + Copy {
//     move |s| {
//         p1(s)?;
//         p2(s)
//     }
// }

// fn right<S, R1, R2>(p1: impl for<'a>FnOnce(&mut Yo<'a, S>) -> Option<R1> + Copy, p2: impl for<'a>FnOnce(&mut Yo<'a, S>) -> Option<R2> + Copy) -> impl for<'a>FnOnce(&mut Yo<'a, S>) -> Option<R2> + Copy {
//     move |s| {
//         p1(s)?;
//         p2(s)
//     }
// }

fn or<'a, S, B: BorrowMut<S>, R>(p1: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R> + Copy, p2: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R> + Copy) -> impl FnOnce(&mut Yo<'a, S, B>) -> Option<R> + Copy {
    move |s| {
        let pos = s.input.clone();
        if let a@Some(_) = p1(s) {
            a
        } else {
            s.input = pos;
            p2(s)
        }
    }
}
//
fn or_diff<'a, S, B: BorrowMut<S>, R1, R2>(p1: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R1> + Copy, p2: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R2> + Copy) -> impl FnOnce(&mut Yo<'a, S, B>) -> Option<()> + Copy {
    move |s| {
        let pos = s.input.clone();
        if let Some(_) = p1(s) {
            Some(())
        } else {
            s.input = pos;
            p2(s)?;
            Some(())
        }
    }
}
//
fn lift_to_state<'a, S, B: BorrowMut<S>, T, R>(f: impl FnOnce(&mut S, T) -> R + Copy, p: impl FnOnce(&mut Yo<'a, S, B>) -> Option<T> + Copy) -> impl FnOnce(&mut Yo<'a, S, B>) -> Option<R> + Copy {
    move |s| {
        let res = p(s)?;
        Some(f(s.user_state.borrow_mut(), res))
    }
}
//
//
pub fn parse<'a, S, B: BorrowMut<S>, R>(p: impl FnOnce(&mut Yo<'a, S, B>) -> Option<R>, input: &'a str, user_state: B) -> (Yo<'a, S, B>, Option<R>) {
    let mut parser_state = Yo { input, user_state, phantom: PhantomData };
    let result = p(&mut parser_state);
    (parser_state, result)
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
