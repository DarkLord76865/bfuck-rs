//! The Brainfuck interpreter.



use crate::code::{Token, TokenStream, STORAGE_SIZE};
use crate::io::{getchar, putchar};



/// Interpret given token stream.
/// # Arguments
/// * token_stream - The [TokenStream] to interpret.
/// # Example
/// ```
/// use bfuck::{process_code, interpret};
///
/// // brainfuck code that prints "Brainfuck"
/// let bf_code = "
/// >++++++[<+++++++++++>-]>+++++++[<+++++++
/// +++++++>-]<->>+++++++++[<+++++++++++>-]>
/// ++++++++[<+++++++++++++>-]<-->>+++++++++
/// +[<+++++++++++>-]<----->>++++++++[<+++++
/// ++++++++>-]<+++>>++++++++++[<+++++++++++
/// >-]>++++++++++[<++++++++++++>-]<------>>
/// +++++++++[<+++++++++++++>-]<<<<<<<<<.>>>
/// >>>>.<<<<<<.>>>.>>.<<<.>>>>>.<<<<<<.>>>.
/// ";
///
/// interpret(process_code(bf_code).unwrap());
/// ```
pub fn interpret(token_stream: TokenStream) {
    let mut ins_ptr = 0;
    let mut data_ptr = 0;
    let mut storage = [0_u8; STORAGE_SIZE];
    let mut loop_stack = Vec::new();

    while ins_ptr < token_stream.len() {
        match token_stream[ins_ptr] {
            Token::Add(n) => storage[data_ptr] = storage[data_ptr].wrapping_add(n),
            Token::Mov(n) => {
                data_ptr += n;
                if data_ptr >= STORAGE_SIZE {
                    data_ptr -= STORAGE_SIZE;
                }
            },
            Token::Input => storage[data_ptr] = getchar(),
            Token::Output => putchar(storage[data_ptr]),
            Token::OpenBr => {
                if storage[data_ptr] == 0 {  // skip the loop
                    let mut nested_loops = 1;
                    while token_stream[ins_ptr] != Token::CloseBr || nested_loops != 0 {
                        ins_ptr += 1;
                        match token_stream[ins_ptr] {
                            Token::OpenBr => nested_loops += 1,
                            Token::CloseBr => nested_loops -= 1,
                            _ => (),
                        }
                    }
                } else {
                    loop_stack.push(ins_ptr);
                }
            },
            Token::CloseBr => {
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
