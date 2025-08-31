use std::string::String;

use crate::{core::{Parser, StrParser}, findbyte::get_byte_pos, json::{string_element_finder, JsonParser}, slicelike::SliceLike};

fn parse_escaped(mut input: &str) -> Option<(String, &str)> {
    let mut result = String::new();
    input = input.strip_prefix('"')?;

    loop {
        let (b, pos) = get_byte_pos(input, string_element_finder())?;
        result.push_str(&input[..pos]);
        input = &input[pos + 1..];

        match b {
            b'"' => {
                return Some((result, input));
            }

            b'\\' => {
                // Parse escaped
                let (first, mut rest) = input.slice_first_if(|_| true)?;
                match first {
                    '\\' | '"' | '/' => result.push(first),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0C'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'u' => {
                        // Unicode routine
                        let unicode;
                        (unicode, rest) = rest.split_at_checked(4)?;
                        let scalar = u16::from_str_radix(unicode, 16).ok()?;
                        let character = char::from_u32(scalar as u32)?;
                        result.push(character);

                    }
                    _ => {
                        return None
                    }
                }

                input = rest;
            }

            _ => {
                return None;
            }
        }
    }
}

/// A parser for JSON strings with escaped characters translated to their
/// corresponding UTF-8 characters.
pub const fn escaped_string_parser<'a, S>() -> impl StrParser<'a, String, S> {
    create_parser!(s, {
        let res;
        (res, s.input) = parse_escaped(s.input)?;
        Some(res)
    })
}

#[cfg(test)]
mod tests {
    use std::string::ToString;

    use crate::{core::ParserExtNoState, json::escaped_string_parser};


    #[test]
    fn formatted_string() {
        let input = r#""some text \"in quotes\"\nnext tab\t and euro \u20AC""#;

        let res = escaped_string_parser().parse(input);
        assert_eq!(res.result, Some("some text \"in quotes\"\nnext tab\t and euro €".to_string()));
    }
}