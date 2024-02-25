## anpa-rs

A generic monadic parser combinator library loosely based on [anpa](https://github.com/habbbe/anpa) which in turn is inspired by Haskell's parsec.

### Features

All parsers and combinators, with few exceptions (`many_to_vec`, `many_to_map`,
`many_to_map_ordered`), are allocation free.

### Examples

See the provided test parsers
- [JSON parser](src/lib/json.rs): JSON DOM parser. It's only ~30 LOC and gives a good
  overview on how to use the library, including recursive parsers.
- [SemVer Parser](src/lib/semver.rs): a parser for the SemVer format

### Dependencies

None

### TODO

- Add documentation
- Add support for `Read`
- More extensive test cases
- Properly structure project
