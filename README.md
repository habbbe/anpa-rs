## anpa

A generic monadic parser combinator library based on [anpa](https://github.com/habbbe/anpa) which in turn is inspired by Haskell's parsec.

### Features

All parsers and combinators, with the exceptions below, are allocation free and
can be used with `no_std` when disabling the default features of this crate.

Allocating parsers:
- `many_to_vec`, `many_to_map_ordered`: Require feature "alloc"
- `many_to_map`: Requires feature "std"


### Examples

See the provided test parsers
- [JSON parser](src/lib/json.rs): JSON DOM parser. It's only ~30 LOC and gives a good
  overview on how to use the library, including recursive parsers.
- [SemVer Parser](src/lib/semver.rs): a parser for the SemVer format

These parsers can be enabled using the features "json" and "semver" respectively.

### Dependencies

None

### TODO

- Add support for `Read`
- More extensive test cases

### License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
