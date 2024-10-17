#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
pub mod macros;
pub mod parsers;
pub mod number;
pub mod charlike;
pub mod core;
pub mod combinators;
pub mod slicelike;
pub mod prefix;
pub mod needle;
pub mod whitespace;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "semver")]
pub mod semver;