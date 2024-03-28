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

    // unsafe block because unchecked methods are used to eliminate bounds checks
    unsafe {
        while ins_ptr < token_stream.len() {
            match *token_stream.get_unchecked(ins_ptr) {
                Token::Add(n) => {
                    *storage.get_unchecked_mut(data_ptr) = storage.get_unchecked(data_ptr).wrapping_add(n)
                },
                Token::Mov(n) => {
                    data_ptr += n;
                    if data_ptr >= STORAGE_SIZE {
                        data_ptr -= STORAGE_SIZE;
                    }
                },
                Token::Input => {
                    *storage.get_unchecked_mut(data_ptr) = getchar()
                },
                Token::Output => {
                    putchar(*storage.get_unchecked(data_ptr));
                },
                Token::OpenBr(jmp) => {
                    // skip the loop if the current cell is 0
                    if *storage.get_unchecked(data_ptr) == 0 {
                        ins_ptr += jmp;
                    }
                },
                Token::CloseBr(jmp) => {
                    // return to the start of the loop if the current cell is not 0
                    if *storage.get_unchecked(data_ptr) != 0 {
                        ins_ptr -= jmp;
                    }
                },
                Token::ClearCell => {
                    *storage.get_unchecked_mut(data_ptr) = 0
                },
            }
            ins_ptr += 1;
        }
    }
}
