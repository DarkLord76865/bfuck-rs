//! Module containing the Error enum for errors that can occur in this crate.


use std::error::Error as StdError;
use std::fmt::Display;


/// Error enum for errors that can occur in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The input contains a non-ASCII character.
    NonASCIIChar(char, usize, usize),
    /// Unmatched open bracket.
    UnmatchedOpenBr(usize, usize),
    /// Unmatched close bracket.
    UnmatchedCloseBr(usize, usize),
    /// The current platform is not supported for JIT-compilation, use interpreter instead.
    UnsupportedPlatformJIT,
    /// The target platform is not supported.
    UnsupportedTarget,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NonASCIIChar(c, row, col) => write!(f, "Non-ASCII character '{}' at line {}, column {}.", c, row, col),
            Error::UnmatchedOpenBr(row, col) => write!(f, "Unmatched '[' at line {}, column {}.", row, col),
            Error::UnmatchedCloseBr(row, col) => write!(f, "Unmatched ']' at line {}, column {}.", row, col),
            Error::UnsupportedPlatformJIT => write!(f, "The current platform is not supported for JIT-compilation, use interpreter instead."),
            Error::UnsupportedTarget => write!(f, "The target platform is not supported."),
        }
    }
}
impl StdError for Error {}
