use std::io::{self, Read, Write};
use std::slice;

pub extern "C" fn getchar() -> u8 {
    io::stdout().flush().unwrap();  // flush the output buffer, before reading input
    let mut read_char = 0;
    io::stdin().lock().read_exact(slice::from_mut(&mut read_char)).expect("Error while reading input!");
    read_char
}

pub extern "C" fn putchar(char_code: u8) {
    print!("{}", char_code as char);
}
