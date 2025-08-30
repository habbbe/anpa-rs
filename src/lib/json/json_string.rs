use std::string::String;

use crate::{core::Parser, findbyte::until_byte, json::string_element_finder, slicelike::SliceLike};

pub const fn escaped_string_parser<'a, S>() -> impl Parser<&'a str, String, S> {
    create_parser!(s, {
        let mut result = String::new();
        skip!('"')(s)?;

        loop {
            let (b, consumed) = until_byte(string_element_finder(), false, true)(s)?;
            result.push_str(consumed);

            match b {
                b'"' => {
                    return Some(result);
                }

                b'\\' => {
                    // Parse escaped
                    let (first, mut rest) = s.input.slice_first_if(|_| true)?;
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

                    s.input = rest;
                }

                _ => {
                    return None;
                }
            }
        }
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