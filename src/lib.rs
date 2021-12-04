#![doc = include_str!("../README.md")]
#![no_std]
#![allow(dead_code)]

pub mod inout;
pub mod input;
pub mod output;

/// Errors
#[derive(Debug)]
pub enum Error {
    // Pin number not within the allowed range.
    PinOutOfRange,
}

/// Trait to be implemented by any chain to return its length.
pub trait Length {
    /// Returns the total length
    fn len(&self) -> usize;

    /// Checks if length is 0
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
