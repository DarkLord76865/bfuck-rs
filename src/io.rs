//! Implementation of the C putchar and getchar functions in Rust.


use std::io::{self, Read, Write};
use std::slice;


/// Read a single byte from the standard input.
/// # Returns
/// * The byte read from the standard input.
pub extern "C" fn getchar() -> u8 {
    io::stdout().flush().unwrap();  // flush the output buffer before reading input
    let mut read_char = 0;

    loop {
        if let Err(err_kind) = io::stdin().lock().read_exact(slice::from_mut(&mut read_char)).map_err(|e| e.kind()) {
            if err_kind == io::ErrorKind::UnexpectedEof {
                read_char = 0;
            } else {
                eprintln!("Error while reading input: {}", err_kind);
            }
        };
        if read_char != b'\r' {
            break;
        }
    }

    read_char
}

/// Write a single byte to the standard output.
/// # Arguments
/// * `byte` - The byte to be written to the standard output.
pub extern "C" fn putchar(byte: u8) {
    if byte < 128 {
        io::stdout().write_all(&[byte]).unwrap();
    }
}
