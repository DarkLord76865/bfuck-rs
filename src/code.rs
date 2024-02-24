//! Tokenization and optimization of code.


use std::fmt::Write;
use std::process::exit;


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Add(u8),      // +/- (-1 represented using twos complement, here as 255)
    Mov(isize),  // </> (< represented as -1, > represented as +1)
    Input,        // ,
    Output,       // .
    OpenPar,      // [
    ClosePar,     // ]
}

pub type TokenStream = Vec<Token>;
pub const STORAGE_SIZE: usize = 30_000;

pub fn tokenize_code(code: &str) -> TokenStream {
    let mut tokens = Vec::new();
    for (i, line) in code.lines().enumerate() {
        for (j, character) in line.chars().enumerate() {
            match character {
                '+' => tokens.push((Token::Add(1), i + 1, j + 1)),
                '-' => tokens.push((Token::Add(u8::MAX), i + 1, j + 1)),
                '<' => tokens.push((Token::Mov(-1), i + 1, j + 1)),
                '>' => tokens.push((Token::Mov(1), i + 1, j + 1)),
                ',' => tokens.push((Token::Input, i + 1, j + 1)),
                '.' => tokens.push((Token::Output, i + 1, j + 1)),
                '[' => tokens.push((Token::OpenPar, i + 1, j + 1)),
                ']' => tokens.push((Token::ClosePar, i + 1, j + 1)),
                _ => {},  // Ignore all other characters (comments, etc.)
            }
        }
    }

    tokens = optimize_adjacent(tokens);

    if let Err(error_messages) = check_loops(&tokens) {
        eprintln!("{}", error_messages);
        exit(1);
    }

    tokens.into_iter().map(|(token, _, _)| token).collect()
}

fn optimize_adjacent(tokens: Vec<(Token, usize, usize)>) -> Vec<(Token, usize, usize)> {
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
            Some((Token::Mov(n), _, _)) => {
                if let Token::Mov(m) = token.0 {
                    *n = (*n + m) % STORAGE_SIZE as isize;
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

fn check_loops(tokens: &[(Token, usize, usize)]) -> Result<(), String> {
    let mut loop_stack = Vec::new();
    let mut error_messages = String::new();

    for (token, row, col) in tokens.iter() {
        match token {
            Token::OpenPar => loop_stack.push((row, col)),
            Token::ClosePar => match loop_stack.pop() {
                Some(_) => {},
                None => writeln!(&mut error_messages, "Unmatched ']' at line {} and column {}.", row, col).unwrap(),
            }
            _ => {},
        }
    }

    for (row, col) in loop_stack {
        writeln!(&mut error_messages, "Unmatched '[' at line {} and column {}.", row, col).unwrap();
    }

    if error_messages.is_empty() {
        Ok(())
    } else {
        Err(error_messages.trim().to_owned())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_code() {
        let code = "++[>++<,.-]";
        let tokens = tokenize_code(code);
        assert_eq!(tokens, vec![
            Token::Add(2),
            Token::OpenPar,
            Token::Mov(1),
            Token::Add(2),
            Token::Mov(-1),
            Token::Input,
            Token::Output,
            Token::Add(u8::MAX),
            Token::ClosePar,
        ]);
    }

    #[test]
    fn test_optimize_adjacent() {
        let tokens = vec![
            (Token::Add(1), 1, 1),
            (Token::Add(1), 1, 2),
            (Token::Mov(1), 1, 3),
            (Token::Mov(1), 1, 4),
            (Token::Add(1), 1, 5),
            (Token::Add(255), 1, 6),
            (Token::Mov(1), 1, 7),
            (Token::Mov(-1), 1, 8),
        ];
        let optimized_tokens = optimize_adjacent(tokens);
        assert_eq!(optimized_tokens, vec![
            (Token::Add(2), 1, 1),
            (Token::Mov(2), 1, 3),
        ]);
    }
}