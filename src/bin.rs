use crate::Item::{Action, SyntaxError, Info, Separator};
use std::fs::File;
use std::io::{self, BufRead};
use std::time::Instant;
use std::string::FromUtf8Error;
use core::borrow::Borrow;


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
        use anpa::core::{*};
        use anpa::{*};
        use anpa::{parsers::{*}, combinators::{*}};

        let parse_name = until_seq("=");
        let parse_cmd = not_empty(rest());
        let parse_action = right(seq("Com:"), lift!(action, parse_name, parse_cmd));
        let parse_info = right(seq("Info:"), lift!(info, parse_name, parse_cmd));
        let parse_separator = lift!(|_| Item::Separator, seq("Separator"));
        let parse_space = lift!(|_| Item::Space, seq("Space"));
        let parse_error = lift!(syntax_error, rest());
        let parse_item = or!(parse_action, parse_info, parse_separator, parse_space, parse_error);
        let item_to_state = lift_to_state(|x: &mut Vec<_>, y| x.push(y), parse_item);
        let ignore = or_diff!(empty(), seq("#"));
        let state_parser = or_diff!(ignore, item_to_state);

        let mut vec: Vec<Item> = Vec::with_capacity(1000000);
        let now = Instant::now();
        for l in &lines {

            // let r = parse_handrolled(&l);
            // match r {
            //     None => {
            //         println!("No parse");
            //         break
            //     }
            //     Some(Item::Ignore) => {}
            //     Some(res) => vec.push(res)
            // }

            let r = parse_state(state_parser, l.as_str(), &mut vec);
            if r.1.is_none() {
                    println!("No parse");
                    break
            }
        }

        //
        println!("N: {}, in {}ms", vec.len(), now.elapsed().as_micros() as f64 / 1000.0);

        fn in_parens<'a, B>(thing: &'a str) -> impl Parser<&str, &str, B> {
            create_parser!(s, {
                or(seq(thing), right(seq("("), left(in_parens(thing), seq(")"))))(s)
            })
        }

        let p = in_parens("hej");
        let x = "(((((((((hej)))))))))";
        let num = parse(p, x);
        if let Some(res) = num.1 {
            println!("Got the thing!")
        }
    }
}

fn action<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Action {name, com}
}

fn info<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Info { name, com }
}

fn syntax_error<'a>(description: &'a str) -> Item<'a> {
    SyntaxError {description}
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
