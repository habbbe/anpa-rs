use std::string::String;

use crate::{charlike::CharLike, core::StrParser, findbyte::get_byte_pos, json::string_element_finder, prefix::Prefix, slicelike::SliceLike};

trait ToJsonString: SliceLike<RefItem: CharLike> + AsRef<[u8]> {
    fn skip_quote(&self) -> Option<Self>;
    fn to_str(&self) -> Option<&str>;
    fn four_chars(&self) -> Option<(&str, Self)>;
}

impl ToJsonString for &str {
    #[inline]
    fn skip_quote(&self) -> Option<Self> {
        self.strip_prefix('"')
    }

    #[inline]
    fn to_str(&self) -> Option<&str> {
        Some(self)
    }

    #[inline]
    fn four_chars(&self) -> Option<(&str, Self)> {
        self.split_at_checked(4)
    }
}

impl ToJsonString for &[u8] {
    #[inline]
    fn skip_quote(&self) -> Option<Self> {
        b'"'.skip_prefix(self)
    }

    #[inline]
    fn to_str(&self) -> Option<&str> {
        str::from_utf8(self).ok()
    }

    #[inline]
    fn four_chars(&self) -> Option<(&str, Self)> {
        let (unicode, rest) = self.split_at_checked(4)?;
        Some((str::from_utf8(unicode).ok()?, rest))
    }
}

fn parse_escaped<I: ToJsonString>(mut input: I) -> Option<(String, I)> {
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
                        let (unicode, new_rest) = rest.four_chars()?;
                        let scalar = u16::from_str_radix(unicode, 16).ok()?;
                        let character = char::from_u32(scalar as u32)?;
                        result.push(character);
                        rest = new_rest;
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