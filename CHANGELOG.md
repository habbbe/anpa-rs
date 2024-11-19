# Changelog

## 0.8.0

### New features

- New parser `find_byte` used for searching for bytes in arch native
  chunk sizes (e.g. 8 bytes on 64-bit).
- New parser `until_byte` which works similarly to `find_byte` but
  returns the consumed input instead of the matching byte itself.