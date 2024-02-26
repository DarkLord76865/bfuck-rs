//! Module containing the Error enum for errors that can occur in this crate.


use std::error::Error as StdError;
use std::fmt::Display;


/// Error enum for errors that can occur in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Unmatched open bracket.
    UnmatchedOpenBr(usize, usize),
    /// Unmatched close bracket.
    UnmatchedCloseBr(usize, usize),
    /// The current platform is not supported.
    UnsupportedPlatform,
    /// The target platform is not supported.
    UnsupportedTarget,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnmatchedOpenBr(row, col) => write!(f, "Unmatched '[' at line {}, column {}.", row, col),
            Error::UnmatchedCloseBr(row, col) => write!(f, "Unmatched ']' at line {}, column {}.", row, col),
            Error::UnsupportedPlatform => write!(f, "The current platform is not supported."),
            Error::UnsupportedTarget => write!(f, "The target platform is not supported."),
        }
    }
}
impl StdError for Error {}
