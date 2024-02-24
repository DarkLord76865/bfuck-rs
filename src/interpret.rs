use crate::io::{getchar, putchar};
use crate::code::{Token, TokenStream, STORAGE_SIZE};

pub fn interpret(token_stream: TokenStream) {
    let mut ins_ptr = 0;
    let mut data_ptr = 0;
    let mut storage = [0_u8; STORAGE_SIZE];
    let mut loop_stack = Vec::new();

    while ins_ptr < token_stream.len() {
        match token_stream[ins_ptr] {
            Token::Add(n) => storage[data_ptr] = storage[data_ptr].wrapping_add(n),
            Token::Mov(n) => {
                let new_data_ptr = data_ptr as isize + n;
                if n > 0 {
                    if (new_data_ptr as usize) < STORAGE_SIZE {
                        data_ptr = new_data_ptr as usize;
                    } else {
                        data_ptr = new_data_ptr as usize - STORAGE_SIZE;
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if new_data_ptr < 0 {
                        data_ptr = (STORAGE_SIZE as isize + new_data_ptr) as usize;
                    } else {
                        data_ptr = new_data_ptr as usize;
                    }                    
                }
            },
            Token::Input => storage[data_ptr] = getchar(),
            Token::Output => putchar(storage[data_ptr]),
            Token::OpenPar => {
                if storage[data_ptr] == 0 {  // skip the loop
                    let mut nested_loops = 1;
                    while token_stream[ins_ptr] != Token::ClosePar || nested_loops != 0 {
                        ins_ptr += 1;
                        match token_stream[ins_ptr] {
                            Token::OpenPar => nested_loops += 1,
                            Token::ClosePar => nested_loops -= 1,
                            _ => (),
                        }
                    }
                } else {
                    loop_stack.push(ins_ptr);
                }
            },
            Token::ClosePar => {
                if storage[data_ptr] != 0 {
                    ins_ptr = *loop_stack.last().unwrap();  // return to the start of the loop
                } else {
                    loop_stack.pop();  // remove the loop from the stack, continue to the next instruction
                }
            },
        }
        ins_ptr += 1;
    }
}
