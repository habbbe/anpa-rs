use anpa::core::{*};
use anpa::{*};
use anpa::combinators::{*};
use anpa::parsers::{*};
use json::{vec_parser, bool_parse, option_parser, string_parser};
use number::{float, integer};

use std::fs::File;
use std::hint::black_box;
use std::io::{self, BufRead, Read};
use std::process::exit;
use std::time::{Duration, Instant};

fn main() {
    bench_hubb();
    bench_hubb_handrolled();
    bench_semver();
    bench_json();
    bench_json_derive();
    bench_json_derive2();
}

fn read_file(path: &str) -> io::BufReader<File> {
    let Ok(file) = File::open(path) else {
        println!("File \"{}\" not found. Please copy it from the test folder", path);
        exit(1)
    };

    io::BufReader::new(file)
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

fn bench_fun<T>(mut n: usize, mut f: impl FnMut() -> T) -> (Duration, T) {
    let mut best = Duration::MAX;
    let mut r;

    loop {
        let now = Instant::now();
        r = f();
        best = best.min(now.elapsed());
        n = n.saturating_sub(1);
        if n == 0 {
            break;
        }
    }

    (best, r)
}

fn bench_hubb() {
    let parse_name = until!('=');
    let parse_cmd = not_empty(rest());
    let parse_action = right(skip!("Com:"), map!(action, parse_name, parse_cmd));
    let parse_info = right(skip!("Info:"), map!(info, parse_name, parse_cmd));
    let parse_separator = skip!("Separator").map(|_| Item::Separator);
    let parse_space = skip!("Space").map(|_| Item::Space);
    let ignore = or_diff(skip!('#'), empty()).map(|_| Item::Ignore);
    let parse_error = rest().map(syntax_error);
    let parse_item = or!(parse_action, parse_info, parse_separator, parse_space, ignore, parse_error);
    let parser = lift_to_state(|x: &mut Vec<_>, y| if !matches!(y, Item::Ignore) { x.push(y) }, parse_item);

    let lines: Vec<String> = read_file("hubb").lines().map(Result::unwrap).collect();
    let mut vec: Vec<Item> = Vec::with_capacity(lines.len());

    let (d, _) = bench_fun(10000, || {
        for _ in 0..50 {
            vec.clear();
            for l in &lines {
                let r = parser.parse_state(l, &mut vec);
                if r.result.is_none() {
                    println!("No parse");
                    break
                }
            }
        }
    });

    println!("Hubb: N: {}, in {}us (anpa)", vec.len(), d.as_nanos() as f64 / 1000.0);
}

fn bench_hubb_handrolled() {
    let lines: Vec<String> = read_file("hubb").lines().map(Result::unwrap).collect();
    let mut vec: Vec<Item> = Vec::with_capacity(lines.len());

    let (d, _) = bench_fun(10000, || {
        for _ in 0..50 {
            vec.clear();
            for l in &lines {
                let r = parse_handrolled(l);
                match r {
                    None => {
                        println!("No parse");
                        break
                    }
                    Some(Item::Ignore) => {}
                    Some(res) => vec.push(res)
                }
            }
        }
    });

    println!("Hubb: N: {}, in {}us (handrolled)", vec.len(), d.as_nanos() as f64 / 1000.0);
}

fn bench_json() {
    let mut string = black_box(String::new());
    let _ = read_file("test.json").read_to_string(&mut string);
    let p = json::object_parser::<&str>();

    let (d, _) = bench_fun(10000, || {
        for _ in 0..10 {
            p.parse_default(&string).result.unwrap();
        }
    });

    println!("anpa::json: in {}us", d.as_nanos() as f64 / 1000.0);
}


#[allow(dead_code)]
struct Address {
    street: String,
    zip: String
}

#[allow(dead_code)]
struct Person {
    name: String,
    middle_name: Option<String>,
    age: u8,
    score: f64,

    escape: String,
    member: bool,
    favorite_emojis: String,
    has_paid: bool,
    address: Address
}

#[allow(dead_code)]
struct Db {
    db: Vec<Person>
}

fn address_parser<'a>() -> impl StrParser<'a, Address, String> {
    json_parser_gen_ng!(Address,
        ("street", street, String, string_parser()),
        ("zip", zip, String, string_parser()),
    )
}

fn person_parser<'a>() -> impl StrParser<'a, Person, String> {
    json_parser_gen_ng!(Person,
        ("name", name, String, string_parser()),
        ("middlename", middle_name, String, string_parser(), optional: true),
        ("age", age, u8, integer()),
        ("score", score, f64, float()),
        ("escape\\n", escape, String, string_parser()),
        ("member", member, bool, bool_parse()),
        ("favorite_emojis", favorite_emojis, String, string_parser()),
        ("hasPaid", has_paid, bool, bool_parse()),
        ("address", address, Address, address_parser())
    )
}

fn db_parser<'a>() -> impl StrParser<'a, Db, String> {
    json_parser_gen_ng!(Db,
        ("db", db, Vec<Person>, vec_parser(person_parser())),
    )
}

fn bench_json_derive() {
    let address_parser = json_parser_gen!(|street, zip| Address { street, zip },
        ("street", string_parser()),
        ("zip", string_parser())
    );

    let person_parser =
    json_parser_gen!(|name, middle_name, age, score, escape, member, favorite_emojis, has_paid, address|
                     Person { name, middle_name, age, score, escape, member, favorite_emojis, has_paid, address},
        ("name", string_parser()),
        ("middlename", option_parser(string_parser())),
        ("age", integer()),
        ("score", float()),
        ("escape\\n", string_parser()),
        ("member", bool_parse()),
        ("favorite_emojis", string_parser()),
        ("hasPaid", bool_parse()),
        ("address", address_parser)
                );

    let db_parser = json_parser_gen!(|db| Db { db },
        ("db", vec_parser(person_parser)),);

    let mut string = black_box(String::new());
    let _ = read_file("test.json").read_to_string(&mut string);

    let (d, _) = bench_fun(10000, || {
        for _ in 0..10 {
            parse_default(db_parser, &string).result.unwrap();
        }
    });

    println!("anpa::json_custom: in {}us", d.as_nanos() as f64 / 1000.0);
}

fn bench_json_derive2() {
    let mut string = black_box(String::new());
    let _ = read_file("test.json").read_to_string(&mut string);

    let (d, _) = bench_fun(10000, || {
        for _ in 0..10 {
            let res = db_parser().parse_default(&string);
            if res.result.is_none() {
                println!("{}", res.state.user_state)
            }
        }
    });

    println!("anpa::json_derive: in {}us", d.as_nanos() as f64 / 1000.0);
}

fn bench_semver() {
    use semver;
    let v = black_box("123432134.43213421.5432344-SNAPSHOT+some.build.id");

    let (d, ver) = bench_fun(10000, || {
        for _ in 0..300 {
            semver::parse_inline(v).unwrap();
        }
        semver::parse_inline(v).unwrap()
    });
    println!("SemVer: {:?}, in {}us", ver, d.as_nanos() as f64 / 1000.0);
}

fn action<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Item::Action {name, com}
}

fn info<'a>(name: &'a str, com: &'a str) -> Item<'a> {
    Item::Info { name, com }
}

fn syntax_error(description: &str) -> Item<'_> {
    Item::SyntaxError {description}
}

fn parse_handrolled(input: &'_ str) -> Option<Item<'_>> {
    fn parse_command_tuple(input: &str) -> Option<(&str, &str)> {
        let equal_pos = input.find('=')?;
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
