//! Process Brainfuck code.



use crate::error::Error;



/// The array size used in the Brainfuck program.
pub const STORAGE_SIZE: usize = 30_000;

/// The processed Brainfuck code.
pub type TokenStream = Vec<Token>;

/// The enum representing a parsed Brainfuck command.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Token {
    /// *Addition*
    ///
    /// Add the value (`u8`) to the current cell in the array.
    ///
    /// Subtraction is represented as ```Add(255)``` since `n - 1 = n + 255 (mod 256)`.
    ///
    /// Adjacent additions are merged.
    Add(u8),

    /// *Move*
    ///
    /// Move (increment) data pointer by the value (`usize`).
    ///
    /// Moving in negative direction is represented as ```Mov(STORAGE_SIZE - n)```.
    ///
    /// Moves past `0` wrap to `STORAGE_SIZE - 1`, and moves past `STORAGE_SIZE - 1` wrap to `0`.
    ///
    /// Adjacent moves are merged.
    Move(usize),

    /// *Input*
    ///
    /// Read a byte from `stdin` and store it in the current cell in the array.
    Input,

    /// *Output*
    ///
    /// Print the byte at the current cell in the array to the `stdout`.
    Output,
    
    /// *Open bracket*
    /// 
    /// The start of the loop.
    /// 
    /// `usize` - The distance from the current position to the matching close bracket.
    OpenBr(usize),
    
    /// *Close bracket*
    /// 
    /// The end of the loop.
    ///
    /// `usize` - The distance from the current position to the matching open bracket.
    CloseBr(usize),
    
    /// *Clear cell*
    /// 
    /// Clear (set to 0) the current cell in the array.
    ClearCell,

    /// *Add to*
    ///
    /// Add the value of the current cell to the cell at the given distance.
    /// Negative direction is represented the same as in [Token::Move].
    ///
    /// The current cell is set to 0.
    AddTo(usize),

    /// *Add to copy*
    ///
    /// Add the value of the current cell to the cells at the given distances.
    /// Negative direction is represented the same as in [Token::Move].
    ///
    /// The same as [Token::AddTo], but adds to 2 cells.
    ///
    /// The current cell is set to 0.
    AddToCopy(usize, usize),
}



/// Process raw Brainfuck code into token stream.
/// # Arguments
/// `code` - A string slice that holds the Brainfuck code.
/// # Returns
/// * [TokenStream] - The generated token stream, if [Ok].
/// * [Error] - The encountered error, if [Err].
/// # Errors
/// * `UnmatchedOpenBr(usize, usize)` - There is an unmatched open bracket at the given line and column.
/// * `UnmatchedCloseBr(usize, usize)` - There is an unmatched close bracket at the given line and column.
/// # Example
/// ```
/// use bfuck::code::{process_code, STORAGE_SIZE, Token};
///
/// let code = "<<--[-[>++<,.-]]";
/// let tokens = process_code(code).unwrap();
///
/// assert_eq!(tokens, vec![
///     Token::Move(STORAGE_SIZE - 2),
///     Token::Add(u8::MAX - 1),
///     Token::OpenBr(10),
///     Token::Add(u8::MAX),
///     Token::OpenBr(7),
///     Token::Move(1),
///     Token::Add(2),
///     Token::Move(STORAGE_SIZE - 1),
///     Token::Input,
///     Token::Output,
///     Token::Add(u8::MAX),
///     Token::CloseBr(7),
///     Token::CloseBr(10),
/// ]);
/// ```
pub fn process_code(code: &str) -> Result<TokenStream, Error> {

    // vector of tokens with their locations (line and column) in the original brainfuck code
    let mut tokens_with_loc = Vec::new();

    // generate tokens from brainfuck code
    for (i, line) in code.lines().enumerate() {
        for (j, character) in line.chars().enumerate() {
            match character {
                '+' => tokens_with_loc.push((Token::Add(1), i + 1, j + 1)),
                '-' => tokens_with_loc.push((Token::Add(u8::MAX), i + 1, j + 1)),
                '<' => tokens_with_loc.push((Token::Move(STORAGE_SIZE - 1), i + 1, j + 1)),
                '>' => tokens_with_loc.push((Token::Move(1), i + 1, j + 1)),
                ',' => tokens_with_loc.push((Token::Input, i + 1, j + 1)),
                '.' => tokens_with_loc.push((Token::Output, i + 1, j + 1)),
                '[' => tokens_with_loc.push((Token::OpenBr(0), i + 1, j + 1)),  // set distance to 0 (calculated at the end)
                ']' => tokens_with_loc.push((Token::CloseBr(0), i + 1, j + 1)),  // set distance to 0 (calculated at the end)
                _ => {},  // Ignore all other characters (comments, etc.)
            }
        }
    }

    // merge adjacent tokens
    tokens_with_loc = merge_adjacent(tokens_with_loc);

    // check whether the loops are correct
    check_loops(&tokens_with_loc)?;
    
    // optimize clear cell instruction ([-])
    clear_cell(&mut tokens_with_loc);

    // optimize add to instruction ([->>+<<])
    add_to(&mut tokens_with_loc);
    
    // optimize add to copy instruction ([->>+>+<<<])
    add_to_copy(&mut tokens_with_loc);

    // calculate the distances for the open and close brackets (used in interpreter for jumps)
    calculate_jumps(&mut tokens_with_loc);

    // remove location information and return the token stream
    Ok(tokens_with_loc.into_iter().map(|(token, _, _)| token).collect())
}

/// Merge adjacent addition and move tokens.
/// Adjacent addition is merged by adding the values modulo 256.
/// Adjacent move is merged by adding the values modulo [STORAGE_SIZE].
/// If the merged value becomes no-op, the token is removed.
/// # Arguments
/// `tokens` - A vector of tokens with their locations (line and column) in the original
/// # Returns
/// * Vec<([Token], usize, usize)> - The optimized token stream.
fn merge_adjacent(tokens: Vec<(Token, usize, usize)>) -> Vec<(Token, usize, usize)> {
    let mut optimized_tokens = Vec::new();

    for token in tokens.into_iter() {
        match optimized_tokens.last_mut() {
            Some((Token::Add(n), _, _)) => {
                if let Token::Add(m) = token.0 {
                    *n = n.wrapping_add(m);
                    if *n == 0 {
                        optimized_tokens.pop();
                    }
                } else {
                    optimized_tokens.push(token);
                }
            },
            Some((Token::Move(n), _, _)) => {
                if let Token::Move(m) = token.0 {
                    *n = (*n + m) % STORAGE_SIZE;
                    if *n == 0 {
                        optimized_tokens.pop();
                    }
                } else {
                    optimized_tokens.push(token);
                }
            },
            _ => optimized_tokens.push(token),
        }
    }

    optimized_tokens
}

/// Check if the loops are correct (brackets are matched).
/// # Arguments
/// `tokens` - A slice of tokens with their locations (line and column) in the original
/// # Returns
/// * `()` - If the loops are correct.
/// * [Error] - If the loops are incorrect.
/// # Errors
/// * `UnmatchedOpenBr(usize, usize)` - There is an unmatched open bracket at the given line and column.
/// * `UnmatchedCloseBr(usize, usize)` - There is an unmatched close bracket at the given line and column.
fn check_loops(tokens: &[(Token, usize, usize)]) -> Result<(), Error> {
    let mut loop_stack = Vec::new();

    for (token, row, col) in tokens.iter() {
        match token {
            Token::OpenBr(_) => loop_stack.push((*row, *col)),
            Token::CloseBr(_) => if loop_stack.pop().is_none() {
                return Err(Error::UnmatchedCloseBr(*row, *col));
            }
            _ => {},
        }
    }

    match loop_stack.pop() {
        Some((row, col)) => Err(Error::UnmatchedOpenBr(row, col)),
        None => Ok(()),
    }
}

/// Optimization - Calculate jumps.
/// Calculate the distances for the open and close brackets (used in interpreter for jumps).
/// # Arguments
/// `tokens` - A mutable slice of tokens with their locations (line and column) in the original
fn calculate_jumps(tokens: &mut [(Token, usize, usize)]) {
    let mut loop_stack = Vec::new();

    for i in 0..tokens.len() {
        match tokens[i] {
            (Token::OpenBr(_), _, _) => loop_stack.push(i),
            (Token::CloseBr(_), _, _) => {
                let open_br = loop_stack.pop().unwrap();
                let distance = i - open_br;
                tokens[open_br].0 = Token::OpenBr(distance);
                tokens[i].0 = Token::CloseBr(distance);
            },
            _ => (),
        }
    }
}

/// Optimization - Clear cell.
/// Detects the pattern `[-]` and replaces it with `ClearCell`.
/// Inside the loop there can be any addition/subtraction, cell still gets cleared, eventually.
/// It doesn't matter if there is a loop around the clear cell, it will still be optimized.
fn clear_cell(tokens: &mut Vec<(Token, usize, usize)>) {
    let mut i = tokens.len();
    while let Some(new_i) = i.checked_sub(1) {
        i = new_i;
        if tokens.len() - i < 3 {
            continue;
        }

        if let Token::OpenBr(_) = tokens[i].0 {
            if let Token::Add(_) = tokens[i + 1].0 {
                if let Token::CloseBr(_) = tokens[i + 2].0 {
                    tokens[i].0 = Token::ClearCell;  // replace first token with ClearCell
                    tokens.drain((i + 1)..=(i + 2));  // remove second and third tokens
                    
                    // check if there is a loop (or multiple loops) around the clear cell, if so, remove it
                    while i != 0 && i != tokens.len() - 1 {
                        match tokens[i - 1].0 {
                            Token::OpenBr(_) => {
                                match tokens[i + 1].0 {
                                    Token::CloseBr(_) => {
                                        i -= 1;  // move to the opening bracket position
                                        tokens[i].0 = Token::ClearCell;  // set opening bracket as the ClearCell
                                        tokens.drain((i + 1)..=(i + 2));  // remove old ClearCell and closing bracket
                                    },
                                    _ => break,
                                }
                            },
                            _ => break,
                        }
                    }
                }
            }
        }
    }
}

/// Optimization - Add to.
/// Detects the pattern like `[->>+<<]` and replaces it with `AddTo(2)`.
/// It doesn't matter if there is a loop around the add to, it will still be optimized.
fn add_to(tokens: &mut Vec<(Token, usize, usize)>) {
    let mut i = tokens.len();
    while let Some(new_i) = i.checked_sub(1) {
        i = new_i;
        if tokens.len() - i < 6 {
            continue;
        }

        if let Token::OpenBr(_) = tokens[i].0 {
            if let Token::Add(u8::MAX) = tokens[i + 1].0 {
                if let Token::Move(m1) = tokens[i + 2].0 {
                    if let Token::Add(1) = tokens[i + 3].0 {
                        if let Token::Move(m2) = tokens[i + 4].0 {
                            if let Token::CloseBr(_) = tokens[i + 5].0 {
                                if (m1 + m2) % STORAGE_SIZE == 0 {
                                    tokens[i].0 = Token::AddTo(m1);  // replace first token with AddTo()
                                    tokens.drain((i + 1)..=(i + 5));  // remove other tokens

                                    // check if there is a loop (or multiple loops) around the add to, if so, remove it
                                    while i != 0 && i != tokens.len() - 1 {
                                        match tokens[i - 1].0 {
                                            Token::OpenBr(_) => {
                                                match tokens[i + 1].0 {
                                                    Token::CloseBr(_) => {
                                                        i -= 1;  // move to the opening bracket position
                                                        tokens[i].0 = tokens[i + 1].0;  // set opening bracket as the AddTo
                                                        tokens.drain((i + 1)..=(i + 2));  // remove old AddTo and closing bracket
                                                    },
                                                    _ => break,
                                                }
                                            },
                                            _ => break,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Optimization - Add to copy.
/// Detects the pattern like `[->>+>+<<<]` and replaces it with `AddToCopy(2, 3)`.
/// It doesn't matter if there is a loop around the add to copy, it will still be optimized.
fn add_to_copy(tokens: &mut Vec<(Token, usize, usize)>) {
    let mut i = tokens.len();
    while let Some(new_i) = i.checked_sub(1) {
        i = new_i;
        if tokens.len() - i < 8 {
            continue;
        }

        if let Token::OpenBr(_) = tokens[i].0 {
            if let Token::Add(u8::MAX) = tokens[i + 1].0 {
                if let Token::Move(m1) = tokens[i + 2].0 {
                    if let Token::Add(1) = tokens[i + 3].0 {
                        if let Token::Move(m2) = tokens[i + 4].0 {
                            if let Token::Add(1) = tokens[i + 5].0 {
                                if let Token::Move(m3) = tokens[i + 6].0 {
                                    if let Token::CloseBr(_) = tokens[i + 7].0 {
                                        if ((m1 + m2) % STORAGE_SIZE + m3) % STORAGE_SIZE == 0 {
                                            tokens[i].0 = Token::AddToCopy(m1, (m1 + m2) % STORAGE_SIZE);  // replace first token with AddToCopy()
                                            tokens.drain((i + 1)..=(i + 7));  // remove other tokens

                                            // check if there is a loop (or multiple loops) around the add to copy, if so, remove it
                                            while i != 0 && i != tokens.len() - 1 {
                                                match tokens[i - 1].0 {
                                                    Token::OpenBr(_) => {
                                                        match tokens[i + 1].0 {
                                                            Token::CloseBr(_) => {
                                                                i -= 1;  // move to the opening bracket position
                                                                tokens[i].0 = tokens[i + 1].0;  // set opening bracket as the AddToCopy
                                                                tokens.drain((i + 1)..=(i + 2));  // remove old AddToCopy and closing bracket
                                                            },
                                                            _ => break,
                                                        }
                                                    },
                                                    _ => break,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_code() {
        //! Test the process_code function.
        
        let code = "++[>++<,.-]";
        let tokens = process_code(code).unwrap();
        assert_eq!(tokens, vec![
            Token::Add(2),
            Token::OpenBr(7),
            Token::Move(1),
            Token::Add(2),
            Token::Move(STORAGE_SIZE - 1),
            Token::Input,
            Token::Output,
            Token::Add(u8::MAX),
            Token::CloseBr(7),
        ]);
    }

    #[test]
    fn test_merge_adjacent() {
        //! Test the merge_adjacent function.
        
        let tokens = vec![
            (Token::Add(1), 1, 1),
            (Token::Add(1), 1, 2),
            (Token::Move(1), 1, 3),
            (Token::Move(1), 1, 4),
            (Token::Add(1), 1, 5),
            (Token::Add(255), 1, 6),
            (Token::Move(1), 1, 7),
            (Token::Move(STORAGE_SIZE - 1), 1, 8),
        ];
        let optimized_tokens = merge_adjacent(tokens);

        assert_eq!(optimized_tokens, vec![
            (Token::Add(2), 1, 1),
            (Token::Move(2), 1, 3),
        ]);
    }

    #[test]
    fn test_check_loops() {
        //! Test the check_loops function.
        
        let tokens = vec![
            (Token::OpenBr(1), 1, 1),
            (Token::CloseBr(1), 1, 2),
            (Token::OpenBr(3), 1, 3),
            (Token::OpenBr(1), 1, 4),
            (Token::CloseBr(1), 1, 5),
            (Token::CloseBr(3), 1, 6),
        ];
        assert_eq!(check_loops(&tokens), Ok(()));

        let tokens = vec![
            (Token::OpenBr(1), 1, 1),
            (Token::CloseBr(1), 1, 2),
            (Token::CloseBr(0), 1, 3),
        ];
        assert_eq!(check_loops(&tokens), Err(Error::UnmatchedCloseBr(1, 3)));

        let tokens = vec![
            (Token::OpenBr(0), 1, 1),
            (Token::OpenBr(1), 1, 2),
            (Token::CloseBr(1), 1, 3),
        ];
        assert_eq!(check_loops(&tokens), Err(Error::UnmatchedOpenBr(1, 1)));
    }
    
    #[test]
    fn test_calculate_jumps() {
        //! Test the calculate_jumps function.
        
        let mut tokens = vec![
            (Token::OpenBr(0), 1, 1),
            (Token::OpenBr(0), 1, 2),
            (Token::OpenBr(0), 1, 3),
            (Token::CloseBr(0), 1, 4),
            (Token::CloseBr(0), 1, 5),
            (Token::CloseBr(0), 1, 6),
        ];
        
        calculate_jumps(&mut tokens);
        
        assert_eq!(tokens, vec![
            (Token::OpenBr(5), 1, 1),
            (Token::OpenBr(3), 1, 2),
            (Token::OpenBr(1), 1, 3),
            (Token::CloseBr(1), 1, 4),
            (Token::CloseBr(3), 1, 5),
            (Token::CloseBr(5), 1, 6),
        ]);
    }
    
    #[test]
    fn test_clear_cell() {
        //! Test the clear_cell function.
        
        // [-]
        let mut tokens = vec![
            (Token::OpenBr(2), 1, 1),
            (Token::Add(u8::MAX), 1, 2),
            (Token::CloseBr(2), 1, 3),
        ];
        clear_cell(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::ClearCell, 1, 1),
        ]);

        // [+]
        let mut tokens = vec![
            (Token::OpenBr(2), 1, 1),
            (Token::Add(1), 1, 2),
            (Token::CloseBr(2), 1, 3),
        ];
        clear_cell(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::ClearCell, 1, 1),
        ]);

        // [+++...+++]
        let mut tokens = vec![
            (Token::OpenBr(2), 1, 1),
            (Token::Add(133), 1, 2),
            (Token::CloseBr(2), 1, 3),
        ];
        clear_cell(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::ClearCell, 1, 1),
        ]);

        // [[[++]]]
        let mut tokens = vec![
            (Token::OpenBr(4), 1, 1),
            (Token::OpenBr(2), 1, 2),
            (Token::Add(2), 1, 3),
            (Token::CloseBr(2), 1, 4),
            (Token::CloseBr(4), 1, 5),
        ];
        clear_cell(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::ClearCell, 1, 1),
        ]);
        
        // >[-]<[[-]]
        let mut tokens = vec![
            (Token::Move(1), 1, 1),
            (Token::OpenBr(2), 1, 2),
            (Token::Add(u8::MAX), 1, 3),
            (Token::CloseBr(2), 1, 4),
            (Token::Move(STORAGE_SIZE - 1), 1, 5),
            (Token::OpenBr(4), 1, 6),
            (Token::OpenBr(2), 1, 7),
            (Token::Add(u8::MAX), 1, 8),
            (Token::CloseBr(2), 1, 9),
            (Token::CloseBr(4), 1, 10),
        ];
        clear_cell(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::Move(1), 1, 1),
            (Token::ClearCell, 1, 2),
            (Token::Move(STORAGE_SIZE - 1), 1, 5),
            (Token::ClearCell, 1, 6),
        ]);
        
        // [[[+]]]-
        let mut tokens = vec![
            (Token::OpenBr(6), 1, 1),
            (Token::OpenBr(4), 1, 2),
            (Token::OpenBr(2), 1, 3),
            (Token::Add(1), 1, 4),
            (Token::CloseBr(2), 1, 5),
            (Token::CloseBr(4), 1, 6),
            (Token::CloseBr(6), 1, 7),
            (Token::Add(u8::MAX), 1, 8),
        ];
        clear_cell(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::ClearCell, 1, 1),
            (Token::Add(u8::MAX), 1, 8),
        ]);
    }
    
    #[test]
    fn test_add_to() {
        //! Test the add_to function.
        
        // [->>+<<]
        let mut tokens = vec![
            (Token::OpenBr(5), 1, 1),
            (Token::Add(u8::MAX), 1, 2),
            (Token::Move(2), 1, 3),
            (Token::Add(1), 1, 4),
            (Token::Move(STORAGE_SIZE - 2), 1, 5),
            (Token::CloseBr(5), 1, 6),
        ];
        add_to(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::AddTo(2), 1, 1),
        ]);
        
        // [-<<<+>>>]
        let mut tokens = vec![
            (Token::OpenBr(5), 1, 1),
            (Token::Add(u8::MAX), 1, 2),
            (Token::Move(STORAGE_SIZE - 3), 1, 3),
            (Token::Add(1), 1, 4),
            (Token::Move(3), 1, 5),
            (Token::CloseBr(5), 1, 6),
        ];
        add_to(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::AddTo(STORAGE_SIZE - 3), 1, 1),
        ]);
        
        // [[[->>+<<]]]
        let mut tokens = vec![
            (Token::OpenBr(9), 1, 1),
            (Token::OpenBr(7), 1, 2),
            (Token::OpenBr(5), 1, 3),
            (Token::Add(u8::MAX), 1, 4),
            (Token::Move(2), 1, 5),
            (Token::Add(1), 1, 6),
            (Token::Move(STORAGE_SIZE - 2), 1, 7),
            (Token::CloseBr(5), 1, 8),
            (Token::CloseBr(7), 1, 9),
            (Token::CloseBr(9), 1, 10),
        ];
        add_to(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::AddTo(2), 1, 1),
        ]);
        
        // >[->>+<<]<
        let mut tokens = vec![
            (Token::Move(1), 1, 1),
            (Token::OpenBr(5), 1, 2),
            (Token::Add(u8::MAX), 1, 3),
            (Token::Move(2), 1, 4),
            (Token::Add(1), 1, 5),
            (Token::Move(STORAGE_SIZE - 2), 1, 6),
            (Token::CloseBr(5), 1, 7),
            (Token::Move(STORAGE_SIZE - 1), 1, 8),
        ];
        add_to(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::Move(1), 1, 1),
            (Token::AddTo(2), 1, 2),
            (Token::Move(STORAGE_SIZE - 1), 1, 8),
        ]);
    }
    
    #[test]
    fn test_add_to_copy() {
        //! Test the add_to_copy function.
        
        // [->>+>+<<<]
        let mut tokens = vec![
            (Token::OpenBr(7), 1, 1),
            (Token::Add(u8::MAX), 1, 2),
            (Token::Move(2), 1, 3),
            (Token::Add(1), 1, 4),
            (Token::Move(1), 1, 5),
            (Token::Add(1), 1, 6),
            (Token::Move(STORAGE_SIZE - 3), 1, 7),
            (Token::CloseBr(7), 1, 8),
        ];
        add_to_copy(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::AddToCopy(2, 3), 1, 1),
        ]);
        
        // [-<<<+>>>>+<]
        let mut tokens = vec![
            (Token::OpenBr(7), 1, 1),
            (Token::Add(u8::MAX), 1, 2),
            (Token::Move(STORAGE_SIZE - 3), 1, 3),
            (Token::Add(1), 1, 4),
            (Token::Move(4), 1, 5),
            (Token::Add(1), 1, 6),
            (Token::Move(STORAGE_SIZE - 1), 1, 7),
            (Token::CloseBr(7), 1, 8),
        ];
        add_to_copy(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::AddToCopy(STORAGE_SIZE - 3, 1), 1, 1),
        ]);
        
        // [[[->>+>>+<<<<]]]
        let mut tokens = vec![
            (Token::OpenBr(11), 1, 1),
            (Token::OpenBr(9), 1, 2),
            (Token::OpenBr(7), 1, 3),
            (Token::Add(u8::MAX), 1, 4),
            (Token::Move(2), 1, 5),
            (Token::Add(1), 1, 6),
            (Token::Move(2), 1, 7),
            (Token::Add(1), 1, 8),
            (Token::Move(STORAGE_SIZE - 4), 1, 9),
            (Token::CloseBr(7), 1, 10),
            (Token::CloseBr(9), 1, 11),
            (Token::CloseBr(11), 1, 12),
        ];
        add_to_copy(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::AddToCopy(2, 4), 1, 1),
        ]);
        
        // >[->>>>>+>>>>>+<<<<<<<<<<]<
        let mut tokens = vec![
            (Token::Move(1), 1, 1),
            (Token::OpenBr(7), 1, 2),
            (Token::Add(u8::MAX), 1, 3),
            (Token::Move(5), 1, 4),
            (Token::Add(1), 1, 5),
            (Token::Move(5), 1, 6),
            (Token::Add(1), 1, 7),
            (Token::Move(STORAGE_SIZE - 10), 1, 8),
            (Token::CloseBr(7), 1, 9),
            (Token::Move(STORAGE_SIZE - 1), 1, 10),
        ];
        add_to_copy(&mut tokens);
        assert_eq!(tokens, vec![
            (Token::Move(1), 1, 1),
            (Token::AddToCopy(5, 10), 1, 2),
            (Token::Move(STORAGE_SIZE - 1), 1, 10),
        ]);
    }
}
