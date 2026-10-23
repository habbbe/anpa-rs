# Changelog

## 0.13.0

### Breaking changes
- Signature change of NumLike
- Signature change of FloatLike
- `integer_internal` is no longer public. `integer_custom` with the new
  configuration described below replaces it.

### New features
- Added possibility to request additional metadata with a parsed integer using
  `IntConfig::with_metadata` together with `integer_custom`. The metadata
  currently includes the number of parsed digits, the sign, and the overflow count.
- Added support for parsing integers with overflow using `integer_custom` and
  `IntConfig::allow_overflow`
  - The number of overflows can be obtained in the metadata if requested.
- Added support for saturation when parsing floats using `float_custom` and
  `FloatConfig::saturate`. If enabled, a float on the correct format will never
  fail to parse, but will lose accuracy when its components reach large sizes.
- Added new combinator `hint_type`, that provides a convenient way to hint the
  compiler about the output type of a parser.

### Fixes
- Raw integer parsers now work on slices larger than integer type.
- Allow scientific notation and disallow leading zero for numbers in JSON parser.
- Prevent successful incomplete parses for JSON string and object.
- Disallow multi-digit all-zero components in semver.
- Don't return partial results in `float` if decimal or exponent fails to parse.
- Return negative zero for `float` if parsing e.g. "-0.0".
- Return zero instead of NaN when parsing zero followed by large exponent in `float`.
- `float` parser can now fit a full `usize` in its integer component.

### Optimizations
- Integer parser now pre-calculates number of digits that can be safely parsed
  without using checked arithmetic

## 0.12.0

### Breaking changes

- Remove parser `integer_signed`.
    - `integer` now infers the signedness from the type
    - It is still possible to force unsigned by using `IntConfig::unsigned`
      with `integer_custom`.

### New features
- New parsers `raw_le_integer`, `raw_be_integer`, `raw_ne_integer` for parsing
  raw integers from byte slices.

### Optimizations
- When parsing integers from `&str`, use byte slices when traversing input.

### Fixes
- Fix builds on no_std by using internal powi function for floats

## 0.11.0

### Breaking changes

- Bump MSRV to 1.88
- Remove `ContiguousBytes` in favor of `AsRef<[u8]>`
- Rename:
    - `integer_checked` => `integer`
    - `integer_signed_checked` => `integer_signed`
    - `float_checked` => `float`

 - Unchecked number parsers removed. Functionality still available as per below.

### New features
 - Add `integer_custom` and `float_custom` parsers for configurable parsing
   of numbers, e.g. unchecked, leading zeros, changing base, etc.

### New features
- New variadic parser `bind!`
- New macro `choose_raw!` that functions like `choose!` but with a value
  instead of a parser.
- Add posibility to use & to combine `ByteFinder`s
- Added new parse function `parse_state_owned` that takes ownership
  of the user state while parsing.
- Added new parse function `parse_default` that performs a parse using
  an owned default (via `Default`) user state.

## 0.10.0

### New features

- Feature "alloc"
    - Required for `many_to_vec` and `many_to_map_ordered` (no requirement on "std")
- Add extension functions for starting a parse. Instead of writing
  e.g. `parse(p, input)` you can write `p.parse(input)`.
- Made all parsers and combinators `const`. Note that parsing is _not_ `const` (yet).
- Made Utf8Whitespace implement Prefix.
- New parser `fold_state`, that uses the user state instead of a provided accumulator.
- `many_to_array` now supports `BorrowMut<[T, N]>` in initializer.

## 0.9.0

### New features

- `item_matches!` now supports `matches!` like syntax for the patterns,
  e.g. `item_matches!('a' | 'b')`

## 0.8.0

### New features

- New parser `find_byte` used for searching for bytes in arch native
  chunk sizes (e.g. 8 bytes on 64-bit).
- New parser `until_byte` which works similarly to `find_byte` but
  returns the consumed input instead of the matching byte itself.
- New parser `chain` that can be used to eliminate left recursion.
- New variant to macro `choose!` that allows for match syntax.
- New macro `item_matches!` used for parsing one item that matches
  any of the arguments.
- New parser `many_to_array` for parsing to an array (usable with no_std).
