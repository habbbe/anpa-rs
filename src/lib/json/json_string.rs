use std::string::String;

use crate::{charlike::CharLike, core::Parser, findbyte::get_byte_pos, json::string_element_finder, prefix::Prefix, slicelike::SliceLike};

pub trait ToJsonString<'a>: SliceLike<RefItem: CharLike> + AsRef<[u8]> {
    fn parse(input: Self) -> Option<(String, Self)>;
    fn skip_quote(self) -> Option<Self>;
    fn to_str(self) -> Option<&'a str>;
    fn four_chars(self) -> Option<(&'a str, Self)>;
}

impl<'a> ToJsonString<'a> for &'a str {
    fn parse(input: Self) -> Option<(String, Self)> {
        parse_escaped(input)
    }

    fn skip_quote(self) -> Option<Self> {
        '"'.skip_prefix(self)
    }

    fn to_str(self) -> Option<&'a str> {
        Some(self)
    }

    fn four_chars(self) -> Option<(&'a str, Self)> {
        self.split_at_checked(4)
    }
}

impl<'a> ToJsonString<'a> for &'a [u8] {
    fn parse(input: Self) -> Option<(String, Self)> {
        parse_escaped(input)
    }

    fn skip_quote(self) -> Option<Self> {
        b'"'.skip_prefix(self)
    }

    fn to_str(self) -> Option<&'a str> {
        str::from_utf8(self).ok()
    }

    fn four_chars(self) -> Option<(&'a str, Self)> {
        let (unicode, rest) = self.split_at_checked(4)?;
        Some((unicode.to_str()?, rest))
    }
}

fn parse_escaped<'a, I: ToJsonString<'a>>(mut input: I) -> Option<(String, I)> {
    let mut result = String::new();
    input = input.skip_quote()?;

    loop {
        let (b, pos) = get_byte_pos(input, string_element_finder())?;
        result.push_str(input.slice_to(pos).to_str()?);
        input = input.slice_from(pos + true.into());

        match b {
            b'"' => {
                return Some((result, input));
            }

            b'\\' => {
                // Parse escaped
                let (first, mut rest) = input.slice_first_if(|_| true)?;
                let first = first.as_char();
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
                        (unicode, rest) = rest.four_chars()?;
                        let scalar = u16::from_str_radix(unicode, 16).ok()?;
                        let character = char::from_u32(scalar as u32)?;
                        result.push(character);
                    }
                    _ => break
                }

                input = rest;
            }

            _ => break
        }
    }

    None
}

/// A parser for JSON strings with escaped characters translated to their
/// corresponding UTF-8 characters.
pub const fn escaped_string_parser<'a, I: ToJsonString<'a>, S>() -> impl Parser<I, String, S> {
    create_parser!(s, {
        let res;
        (res, s.input) = I::parse(s.input)?;
        Some(res)
    })
}

#[cfg(test)]
mod tests {
    use std::string::ToString;

    use crate::{core::ParserExtNoState, json::escaped_string_parser};

    const RAW_STRING: &str = r#""some text \"in quotes\"\nnext tab\t and euro \u20AC""#;
    const RESULT_STRING: &str = "some text \"in quotes\"\nnext tab\t and euro €";

    #[test]
    fn formatted_string() {
        let res = escaped_string_parser().parse(RAW_STRING);
        assert_eq!(res.result, Some(RESULT_STRING.to_string()));
    }

    #[test]
    fn formatted_bytes() {
        let res = escaped_string_parser().parse(RAW_STRING.as_bytes());
        assert_eq!(res.result, Some(RESULT_STRING.to_string()));
    }
}
