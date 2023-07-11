use std::io::{self, Read, Write};
use std::process::exit;
use std::ptr;
use std::slice;
use super::Storage;


pub fn interpret(brainfuck_code: String) {
    let mut storage = Storage::default();
    let mut ptr: usize = 0;
    let mut ins_ptr: usize = 0;
    let stdin = io::stdin();

    let brainfuck_commands: Vec<char> = brainfuck_code.chars().collect();
    let mut loop_stack: Vec<usize> = Vec::new();

    while ins_ptr < brainfuck_commands.len() {
        match brainfuck_commands[ins_ptr] {
            '+' => {
                storage[ptr] = storage[ptr].wrapping_add(1_u8);
            },
            '-' => {
                storage[ptr] = storage[ptr].wrapping_add_signed(-1_i8);
            },
            '>' => {
                ptr += 1;
            },
            '<' => {
                if ptr == 0 {
                    eprintln!("Data pointer index out of bounds!");
                    exit(1);
                } else {
                    ptr -= 1;
                }
            },
            '.' => {
                print!("{}", storage[ptr] as char);
            },
            ',' => {
                io::stdout().flush().unwrap();
                stdin.lock().read_exact(unsafe {slice::from_raw_parts_mut(ptr::addr_of_mut!(storage[ptr]), 1)}).expect("Error while reading input!");
            },
            '[' => {
                if storage[ptr] != 0 {
                    loop_stack.push(ins_ptr);
                } else {
                    let mut count: usize = 0;
                    ins_ptr += 1;
                    while brainfuck_commands[ins_ptr] != ']' || count != 0 {
                        if brainfuck_commands[ins_ptr] == '[' {
                            count += 1;
                        } else if brainfuck_commands[ins_ptr] == ']' {
                            count -= 1;
                        }
                        ins_ptr += 1;
                    }
                }
            },
            ']' => {
                if storage[ptr] != 0 {
                    ins_ptr = loop_stack[loop_stack.len() - 1];
                } else {
                    loop_stack.pop();
                }
            },
            _ => {
                eprintln!("Unknown command!");
                exit(1);
            }
        }
        ins_ptr += 1;
    }
}
