use anpa::core::{*};
use anpa::json::JsonValue;
use anpa::semver::AnpaVersion;
use anpa::{*};
use anpa::combinators::{*};
use anpa::parsers::{*};

use std::fs::File;
use std::io::{self, BufRead, Read};
use std::process::exit;
use std::time::Instant;

fn main() {
    bench_json();
    bench_hubb();
    bench_hubb_handrolled();
    bench_semver();
}

#[derive(Debug)]
#[allow(dead_code)]
enum Item<'a> {
    Action { name: &'a str, com: &'a str },
    Info { name: &'a str, com: &'a str },
    Separator,
    Space,
    Ignore,
    SyntaxError { description: &'a str }
}

fn read_file(path: &str) -> io::BufReader<File> {
    let Ok(file) = File::open(path) else {
        println!("File \"{}\" not found. Please copy it from the test folder", path);
        exit(1)
    };

    io::BufReader::new(file)
}

fn bench_hubb() {
    let parse_name = until_seq("=");
    let parse_cmd = not_empty(rest());
    let parse_action = right!(seq("Com:"), lift!(action, parse_name, parse_cmd));
    let parse_info = right!(seq("Info:"), lift!(info, parse_name, parse_cmd));
    let parse_separator = seq("Separator").map(|_| Item::Separator);
    let parse_space = seq("Space").map(|_| Item::Space);
    let parse_error = lift!(syntax_error, rest());
    let parse_item = or!(parse_action, parse_info, parse_separator, parse_space, parse_error);
    let item_to_state = lift_to_state(|x: &mut Vec<_>, y| x.push(y), parse_item);
    let ignore = or_diff!(empty(), item('#'));
    let state_parser = or_diff!(ignore, item_to_state);

    let lines: Vec<String> = read_file("hubb").lines().map(Result::unwrap).collect();
    let mut vec: Vec<Item> = Vec::with_capacity(lines.len());
    let now = Instant::now();

    for l in &lines {
        let r = parse_state(state_parser, l, &mut vec);
        if r.1.is_none() {
                println!("No parse");
                break
        }
    }

    println!("Hubb: N: {}, in {}ms (anpa)", vec.len(), now.elapsed().as_micros() as f64 / 1000.0);
}

fn bench_hubb_handrolled() {
    let lines: Vec<String> = read_file("hubb").lines().map(Result::unwrap).collect();
    let mut vec: Vec<Item> = Vec::with_capacity(lines.len());
    let now = Instant::now();

    for l in &lines {
        let r = parse_handrolled(&l);
        match r {
            None => {
                println!("No parse");
                break
            }
            Some(Item::Ignore) => {}
            Some(res) => vec.push(res)
        }
    }
    println!("Hubb: N: {}, in {}ms (handrolled)", vec.len(), now.elapsed().as_micros() as f64 / 1000.0);
}

fn bench_json() {
    let mut string = String::new();
    let _ = read_file("canada.json").read_to_string(&mut string);
    let p = json::object_parser::<&str>();

    let now = Instant::now();
    let res = parse(p, &string);
    match res.1 {
        Some(JsonValue::Dic(dic)) =>
            println!("JSON: N: {}, in {}ms", dic.len(), now.elapsed().as_micros() as f64 / 1000.0),
        _ => println!("No parse"),
    }
}

fn bench_semver() {
    let v = "123432134.43213421.5432344-SNAPSHOT+some.build.id";

    let mut ver = AnpaVersion::<_>::new(0, 0, 0, "", "");
    let now = Instant::now();
    for _ in 0..200000 {
        ver = semver::parse_version_inline(v).unwrap();
    }

    println!("Version: {:?}, in {}ms", ver, now.elapsed().as_micros() as f64 / 1000.0);
}

fn action<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Item::Action {name, com}
}

fn info<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Item::Info { name, com }
}

fn syntax_error<'a>(description: &'a str) -> Item<'a> {
    Item::SyntaxError {description}
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
        Some(action(name, com))
    } else if let Some(rest) = parse_and_get_rest(input, "Info:") {
        let (name, com) = parse_command_tuple(rest)?;
        Some(info(name, com))
    } else if parse_and_get_rest(input, "Separator").is_some() {
        Some(Item::Separator)
    } else if parse_and_get_rest(input, "Space").is_some() {
        Some(Item::Space)
    } else if parse_and_get_rest(input, "#").is_some() || input.is_empty() {
        Some(Item::Ignore)
    } else {
        Some(syntax_error(input))
    }
}