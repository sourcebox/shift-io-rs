#![doc = include_str!("../README.md")]
#![no_std]
#![allow(dead_code)]

pub mod inout;
pub mod input;
pub mod output;

/// Errors
#[derive(Debug)]
pub enum Error {
    // Pin number is not within the allowed range
    PinOutOfRange,
}
