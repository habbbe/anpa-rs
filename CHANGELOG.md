# Changelog

## 0.10.0

### New features

- Add extension functions for starting a parse. Instead of writing
  e.g. `parse(p, input)` you can write `p.parse(input)`.

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
