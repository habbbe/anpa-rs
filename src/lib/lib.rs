#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
pub mod macros;
pub mod parsers;
pub mod number;
pub mod core;
pub mod combinators;
pub mod asciilike;
pub mod slicelike;
pub mod searchee;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "semver")]
pub mod semver;