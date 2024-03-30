//! Convert text to Brainfuck code.



use std::cmp::Ordering;
use std::collections::HashMap;
use crate::error::Error;



/// Generate Brainfuck code that prints the provided text.
/// # Arguments
/// * `text` - The text that Brainfuck code should print.
/// # Returns
/// * [String] - The Brainfuck code, if OK.
/// * [Error] - The error.
/// # Errors
/// * `Error::NonASCIIChar` - If a non-ASCII character is found.
/// # Example
/// ```
/// use bfuck::text::text_2_bf;
///
/// let text = "Brainfuck";
/// let bf_code = text_2_bf(text).unwrap();
///
/// // Brainfuck code that prints "Brainfuck"
/// let expected_code = ">++++++[<+++++++++++>-]>++++++++[<++++++++++++>-]<+>>+++++++++[<+++++++++++>-]>++++++++++[<++++++++++>-]<++>>++++++++[<+++++++++++++>-]<+>>+++++++++[<++++++++++++>-]<->>++++++++++[<+++++++++++>-]>++++++++[<++++++++++++++>-]<++>>+++++++++[<+++++++++++++>-]<<<<<<<<<.>>>>>>>.<<<<<<.>>>.>>.<<<.>>>>>.<<<<<<.>>>.";
///
/// assert_eq!(bf_code, expected_code);
/// ```
pub fn text_2_bf(text: &str) -> Result<String, Error> {
    // convert text to bytes
    // each character is converted to its ASCII value (single byte)
    let bytes = text_2_bytes(text)?;

    // generate the ordered bytes that will be stored in the array
    let mut store_order = bytes.clone();
    store_order.sort();
    store_order.dedup();

    // resulting Brainfuck code
    let mut bf_code = String::new();

    // generate code for storing bytes into array
    bf_code.push_str(&store_bf_bytes(&store_order));

    // generate code for printing bytes from the array
    bf_code.push_str(&print_bf_bytes(&bytes, &store_order, store_order.len()));

    // return Brainfuck code
    Ok(bf_code)
}

/// Converts a string to a vector of bytes.
/// Each character is converted to its ASCII value (single byte).
/// Characters that are not ASCII are not allowed.
/// # Arguments
/// * `text` - The text to convert.
/// # Returns
/// * `Result<Vec<u8>, Error>` - The vector of bytes if the conversion was successful, otherwise an error.
/// # Errors
/// * `Error::NonASCIIChar` - If a non-ASCII character is found.
fn text_2_bytes(text: &str) -> Result<Vec<u8>, Error> {
    let mut line = 1;
    let mut column = 1;
    let mut bytes = Vec::new();

    for c in text.chars() {
        if c.is_ascii() {
            if c != '\r' {
                bytes.push(c as u8);
                if c == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
            }
        } else {
            return Err(Error::NonASCIIChar(c, line, column))
        }
    }

    Ok(bytes)
}

/// Generate Brainfuck code for storing sequence of bytes into array
/// Data pointer is left at the index == bytes.len()
/// # Arguments
/// * `bytes` - The slice of bytes to store into array
/// # Returns
/// * [String] - The Brainfuck code.
fn store_bf_bytes(bytes: &[u8]) -> String {
    let mut store = String::new();

    let fact_table = factor_table();

    for &byte in bytes {
        match fact_table[byte as usize] {
            Some((f1, f2, diff)) => {
                if byte <= 10 {
                    store.push_str(&"+".repeat(byte as usize));
                    store.push('>');
                } else {
                    store.push('>');
                    store.push_str(&"+".repeat(f1 as usize));
                    store.push('[');
                    store.push('<');
                    store.push_str(&"+".repeat(f2 as usize));
                    store.push('>');
                    store.push('-');
                    store.push(']');
                    match diff.cmp(&0) {
                        Ordering::Greater => {
                            store.push('<');
                            store.push_str(&"+".repeat(diff.unsigned_abs() as usize));
                            store.push('>');
                        },
                        Ordering::Less => {
                            store.push('<');
                            store.push_str(&"-".repeat(diff.unsigned_abs() as usize));
                            store.push('>');
                        },
                        Ordering::Equal => (),
                    }
                }
            },
            None => {
                store.push_str(&"+".repeat(byte as usize));
                store.push('>');
            },
        }
    }

    store
}

/// Print out the text_bytes from the sequence of bytes stored in the array
/// # Arguments
/// * `text_bytes` - The slice of bytes to print out.
/// * `store_ord` - The slice of bytes that are stored in the array.
/// * `position` - The position of the data pointer in the array.
/// # Returns
/// * [String] - The Brainfuck code.
fn print_bf_bytes(text_bytes: &[u8], store_ord: &[u8], position: usize) -> String {
    let mut curr_pos = position as isize;
    let mut bf_code = String::new();
    for byte in text_bytes {
        let new_pos = store_ord.iter().position(|&x| x == *byte).unwrap() as isize;
        let diff = new_pos - curr_pos;
        match diff.cmp(&0) {
            Ordering::Greater => {
                bf_code.push_str(&">".repeat(diff.unsigned_abs()));
            },
            Ordering::Less => {
                bf_code.push_str(&"<".repeat(diff.unsigned_abs()));
            },
            Ordering::Equal => (),
        }
        curr_pos = new_pos;
        bf_code.push('.');
    }
    bf_code
}

/// Generate a table of factors and differences for numbers 0 to 255.
/// To get the value of a number, the factors are multiplied and the difference is added.
/// Used to minimize the number of operations in Brainfuck code.
/// None means that the number should be represented just by repeating + commands (numbers less than 10).
fn factor_table() -> Vec<Option<(u8, u8, i8)>> {

    // working table, fill with (index, factors) where factors are set to u8::MAX
    let mut work_table = Vec::with_capacity(u8::MAX as usize + 1);
    for i in 0..=u8::MAX {
        work_table.push((i, [u8::MAX, u8::MAX]));
    }

    // iterate over pairs of factors that when multiplied return n <= u8::MAX
    // we aren't interested in pairs of factors where one of the factors is 1, so we can exclude 1 and u8::MAX
    // also we can just check i < j because we would just get duplicates if we were to check every j for every i
    for i in 2..u8::MAX {
        for j in i..u8::MAX {
            match i.checked_mul(j) {
                Some(mul) => {
                    // if the sum of factors (i, j) is lower than the sum of current factors for n (n = i * j)
                    // set that factors for that n instead
                    // (we want to minimize the sum of factors so that the multiplication loop is shorter in Brainfuck)
                    if (i + j) < (work_table[mul as usize].1[0].saturating_add(work_table[mul as usize].1[1])) {
                        work_table[mul as usize].1 = [i, j];
                    }
                },
                None => break,  // break since further j will just give us a larger number (overflow)
            }
        }
    }

    // remove numbers that still have original factors set (no actual factors were found)
    work_table.retain(|(n, pair)| (*pair != [u8::MAX, u8::MAX]) && (*n > 10));

    // for each number in table, check if it can be represented in a shorter manner by
    // using factors of the previous number and +/- symbols
    // if it can, remove it
    let mut i = 1;
    while i < work_table.len() {
        if work_table[i].1.iter().sum::<u8>() >= work_table[i - 1].1.iter().sum::<u8>() + (work_table[i].0 - work_table[i - 1].0) {
            work_table.remove(i);
        } else {
            i += 1;
        }
    }

    // for each number in table, check if it can be represented in a shorter manner by
    // using factors of the next number and +/- symbols
    // if it can, remove it
    for i in (0..(work_table.len() - 1)).rev() {
        if work_table[i].1.iter().sum::<u8>() >= work_table[i + 1].1.iter().sum::<u8>() + (work_table[i + 1].0 - work_table[i].0) {
            work_table.remove(i);
        }
    }

    // store the available numbers with factors in a map for easier access
    let mut map = HashMap::with_capacity(work_table.len());
    for (i, pair) in work_table {
        map.insert(i, pair);
    }

    // result table, fill with None
    // indices represent numbers
    // values represent the factors of the number, and the third value is the difference between the number and the product of the factors
    // (+/- symbols can be used to get the number from the product of the factors)
    let mut table = vec![None; u8::MAX as usize + 1];

    // for each number, find best way to get it using the factors in the map
    // and store it in the result table, don't calculate for n <= 10 (they are just represented with repeated + commands)
    for i in 11..=u8::MAX {
        let mut high = i;
        let mut low = i;

        while !map.contains_key(&low) && !map.contains_key(&high) {
            high = high.saturating_add(1);
            low = low.saturating_sub(1);
        }

        // search up and down for the closest number with factors

        // get factors of the closest number
        let factors =
            if map.contains_key(&low) {
                if map.contains_key(&high) {
                    if map[&low].iter().sum::<u8>() < map[&high].iter().sum::<u8>() {
                        map[&low]
                    } else {
                        map[&high]
                    }
                } else {
                    map[&low]
                }
            } else {
                map[&high]
            };

        // calculate difference between the number and the product of the factors
        let diff = (i as isize - factors.iter().product::<u8>() as isize) as i8;

        // store factors and difference in the result table
        table[i as usize] = Some((factors[0], factors[1], diff));
    }

    // return the result table
    table
}



#[cfg(test)]
mod tests {
    use super::*;

    /// Checks if a number is prime.
    /// # Arguments
    /// * `num` - The number to check.
    /// # Returns
    /// * `bool` - Whether the number is prime.
    /// * `u64` - The smallest divisor if the number is not prime, otherwise 1.
    fn is_prime(n: u64) -> (bool, u64) {
        assert!(n >= 2, "Number must be greater than or equal to 2.");

        if n == 2 || n == 3 {
            (true, 1)
        } else if n % 2 == 0 {
            (false, 2)
        } else if n % 3 == 0 {
            (false, 3)
        } else {
            for i in (5..=((n as f64).sqrt().floor() as u64)).step_by(6) {
                if n % i == 0 {
                    return (false, i);
                } else if n % (i + 2) == 0 {
                    return (false, i + 2);
                }
            }

            (true, 1)
        }
    }

    #[test]
    fn test_text_2_bf() {
        //! Test the `text_2_bf` function.

        let text = "Brain\nFuck";
        let bf_code = text_2_bf(text).unwrap();

        assert_eq!(bf_code, "++++++++++>>++++++[<+++++++++++>-]>+++++++[<++++++++++>-]>++++++++[<++++++++++++>-]<+>>+++++++++[<+++++++++++>-]>++++++++[<+++++++++++++>-]<+>>+++++++++[<++++++++++++>-]<->>++++++++++[<+++++++++++>-]>++++++++[<++++++++++++++>-]<++>>+++++++++[<+++++++++++++>-]<<<<<<<<<.>>>>>>>.<<<<<.>>.>>.<<<<<<<.>>.>>>>>>>.<<<<<.>>.");
    }

    #[test]
    fn test_text_2_bytes() {
        //! Test the `text_2_bytes` function.

        let text = "Hello, World!";
        let bytes = text_2_bytes(text).unwrap();
        assert_eq!(bytes, vec![72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]);

        let text = "😊Hello, World!";
        let error = text_2_bytes(text).unwrap_err();
        assert_eq!(error, Error::NonASCIIChar('😊', 1, 1));
    }

    #[test]
    fn test_store_bf_bytes() {
        //! Test the `store_bf_bytes` function.

        let mut all_bytes = Vec::with_capacity(u8::MAX as usize + 1);
        for i in 0..=u8::MAX {
            all_bytes.push(i);
        }

        let bf_code = store_bf_bytes(&all_bytes);

        assert_eq!(bf_code, ">+>++>+++>++++>+++++>++++++>+++++++>++++++++>+++++++++>++++++++++>>+++[<++++>-]<->>+++[<++++>-]>+++[<++++>-]<+>>+++[<+++++>-]<->>+++[<+++++>-]>++++[<++++>-]>++++[<++++>-]<+>>+++[<++++++>-]>++++[<+++++>-]<->>++++[<+++++>-]>++++[<+++++>-]<+>>++++[<+++++>-]<++>>++++[<++++++>-]<->>++++[<++++++>-]>+++++[<+++++>-]>+++++[<+++++>-]<+>>++++[<+++++++>-]<->>++++[<+++++++>-]>+++++[<++++++>-]<->>+++++[<++++++>-]>+++++[<++++++>-]<+>>++++[<++++++++>-]>++++[<++++++++>-]<+>>+++++[<+++++++>-]<->>+++++[<+++++++>-]>++++++[<++++++>-]>++++++[<++++++>-]<+>>++++++[<++++++>-]<++>>+++++[<++++++++>-]<->>+++++[<++++++++>-]>++++++[<+++++++>-]<->>++++++[<+++++++>-]>++++++[<+++++++>-]<+>>+++++[<+++++++++>-]<->>+++++[<+++++++++>-]>+++++[<+++++++++>-]<+>>++++++[<++++++++>-]<->>++++++[<++++++++>-]>+++++++[<+++++++>-]>+++++++[<+++++++>-]<+>>+++++++[<+++++++>-]<++>>++++++[<+++++++++>-]<-->>++++++[<+++++++++>-]<->>++++++[<+++++++++>-]>+++++++[<++++++++>-]<->>+++++++[<++++++++>-]>+++++++[<++++++++>-]<+>>+++++++[<++++++++>-]<++>>++++++[<++++++++++>-]<->>++++++[<++++++++++>-]>++++++[<++++++++++>-]<+>>+++++++[<+++++++++>-]<->>+++++++[<+++++++++>-]>++++++++[<++++++++>-]>++++++++[<++++++++>-]<+>>++++++[<+++++++++++>-]>++++++[<+++++++++++>-]<+>>+++++++[<++++++++++>-]<-->>+++++++[<++++++++++>-]<->>+++++++[<++++++++++>-]>++++++++[<+++++++++>-]<->>++++++++[<+++++++++>-]>++++++++[<+++++++++>-]<+>>++++++++[<+++++++++>-]<++>>+++++++[<+++++++++++>-]<-->>+++++++[<+++++++++++>-]<->>+++++++[<+++++++++++>-]>+++++++[<+++++++++++>-]<+>>++++++++[<++++++++++>-]<->>++++++++[<++++++++++>-]>+++++++++[<+++++++++>-]>+++++++++[<+++++++++>-]<+>>+++++++[<++++++++++++>-]<->>+++++++[<++++++++++++>-]>+++++++[<++++++++++++>-]<+>>++++++++[<+++++++++++>-]<-->>++++++++[<+++++++++++>-]<->>++++++++[<+++++++++++>-]>+++++++++[<++++++++++>-]<->>+++++++++[<++++++++++>-]>+++++++++[<++++++++++>-]<+>>+++++++++[<++++++++++>-]<++>>+++++++++[<++++++++++>-]<+++>>++++++++[<++++++++++++>-]<-->>++++++++[<++++++++++++>-]<->>++++++++[<++++++++++++>-]>++++++++[<++++++++++++>-]<+>>+++++++++[<+++++++++++>-]<->>+++++++++[<+++++++++++>-]>++++++++++[<++++++++++>-]>++++++++++[<++++++++++>-]<+>>++++++++++[<++++++++++>-]<++>>++++++++[<+++++++++++++>-]<->>++++++++[<+++++++++++++>-]>++++++++[<+++++++++++++>-]<+>>+++++++++[<++++++++++++>-]<-->>+++++++++[<++++++++++++>-]<->>+++++++++[<++++++++++++>-]>++++++++++[<+++++++++++>-]<->>++++++++++[<+++++++++++>-]>++++++++++[<+++++++++++>-]<+>>++++++++[<++++++++++++++>-]>++++++++[<++++++++++++++>-]<+>>++++++++[<++++++++++++++>-]<++>>+++++++++[<+++++++++++++>-]<-->>+++++++++[<+++++++++++++>-]<->>+++++++++[<+++++++++++++>-]>+++++++++[<+++++++++++++>-]<+>>++++++++++[<++++++++++++>-]<->>++++++++++[<++++++++++++>-]>+++++++++++[<+++++++++++>-]>+++++++++++[<+++++++++++>-]<+>>+++++++++++[<+++++++++++>-]<++>>+++++++++[<++++++++++++++>-]<-->>+++++++++[<++++++++++++++>-]<->>+++++++++[<++++++++++++++>-]>+++++++++[<++++++++++++++>-]<+>>++++++++[<++++++++++++++++>-]>++++++++++[<+++++++++++++>-]<->>++++++++++[<+++++++++++++>-]>+++++++++++[<++++++++++++>-]<->>+++++++++++[<++++++++++++>-]>+++++++++++[<++++++++++++>-]<+>>+++++++++[<+++++++++++++++>-]<->>+++++++++[<+++++++++++++++>-]>+++++++++[<+++++++++++++++>-]<+>>+++++++++[<+++++++++++++++>-]<++>>++++++++++[<++++++++++++++>-]<-->>++++++++++[<++++++++++++++>-]<->>++++++++++[<++++++++++++++>-]>++++++++++[<++++++++++++++>-]<+>>+++++++++++[<+++++++++++++>-]<->>+++++++++++[<+++++++++++++>-]>++++++++++++[<++++++++++++>-]>++++++++++++[<++++++++++++>-]<+>>++++++++++++[<++++++++++++>-]<++>>++++++++++++[<++++++++++++>-]<+++>>++++++++++[<+++++++++++++++>-]<-->>++++++++++[<+++++++++++++++>-]<->>++++++++++[<+++++++++++++++>-]>++++++++++[<+++++++++++++++>-]<+>>+++++++++++[<++++++++++++++>-]<-->>+++++++++++[<++++++++++++++>-]<->>+++++++++++[<++++++++++++++>-]>++++++++++++[<+++++++++++++>-]<->>++++++++++++[<+++++++++++++>-]>++++++++++++[<+++++++++++++>-]<+>>++++++++++++[<+++++++++++++>-]<++>>++++++++++[<++++++++++++++++>-]<->>++++++++++[<++++++++++++++++>-]>++++++++++[<++++++++++++++++>-]<+>>+++++++++[<++++++++++++++++++>-]>+++++++++[<++++++++++++++++++>-]<+>>+++++++++++[<+++++++++++++++>-]<->>+++++++++++[<+++++++++++++++>-]>+++++++++++[<+++++++++++++++>-]<+>>++++++++++++[<++++++++++++++>-]<->>++++++++++++[<++++++++++++++>-]>+++++++++++++[<+++++++++++++>-]>+++++++++++++[<+++++++++++++>-]<+>>+++++++++++++[<+++++++++++++>-]<++>>+++++++++++++[<+++++++++++++>-]<+++>>+++++++++++[<++++++++++++++++>-]<--->>+++++++++++[<++++++++++++++++>-]<-->>+++++++++++[<++++++++++++++++>-]<->>+++++++++++[<++++++++++++++++>-]>+++++++++++[<++++++++++++++++>-]<+>>++++++++++++[<+++++++++++++++>-]<-->>++++++++++++[<+++++++++++++++>-]<->>++++++++++++[<+++++++++++++++>-]>+++++++++++++[<++++++++++++++>-]<->>+++++++++++++[<++++++++++++++>-]>+++++++++++++[<++++++++++++++>-]<+>>+++++++++++++[<++++++++++++++>-]<++>>+++++++++++[<+++++++++++++++++>-]<-->>+++++++++++[<+++++++++++++++++>-]<->>+++++++++++[<+++++++++++++++++>-]>+++++++++++[<+++++++++++++++++>-]<+>>++++++++++[<+++++++++++++++++++>-]<->>++++++++++[<+++++++++++++++++++>-]>++++++++++++[<++++++++++++++++>-]<->>++++++++++++[<++++++++++++++++>-]>++++++++++++[<++++++++++++++++>-]<+>>+++++++++++++[<+++++++++++++++>-]<->>+++++++++++++[<+++++++++++++++>-]>++++++++++++++[<++++++++++++++>-]>++++++++++++++[<++++++++++++++>-]<+>>+++++++++++[<++++++++++++++++++>-]>+++++++++++[<++++++++++++++++++>-]<+>>++++++++++[<++++++++++++++++++++>-]>++++++++++[<++++++++++++++++++++>-]<+>>++++++++++++[<+++++++++++++++++>-]<-->>++++++++++++[<+++++++++++++++++>-]<->>++++++++++++[<+++++++++++++++++>-]>++++++++++++[<+++++++++++++++++>-]<+>>+++++++++++++[<++++++++++++++++>-]<-->>+++++++++++++[<++++++++++++++++>-]<->>+++++++++++++[<++++++++++++++++>-]>++++++++++++++[<+++++++++++++++>-]<->>++++++++++++++[<+++++++++++++++>-]>++++++++++++++[<+++++++++++++++>-]<+>>++++++++++++++[<+++++++++++++++>-]<++>>++++++++++++++[<+++++++++++++++>-]<+++>>++++++++++++[<++++++++++++++++++>-]<-->>++++++++++++[<++++++++++++++++++>-]<->>++++++++++++[<++++++++++++++++++>-]>++++++++++++[<++++++++++++++++++>-]<+>>++++++++++++[<++++++++++++++++++>-]<++>>+++++++++++++[<+++++++++++++++++>-]<-->>+++++++++++++[<+++++++++++++++++>-]<->>+++++++++++++[<+++++++++++++++++>-]>+++++++++++++[<+++++++++++++++++>-]<+>>++++++++++++++[<++++++++++++++++>-]<->>++++++++++++++[<++++++++++++++++>-]>+++++++++++++++[<+++++++++++++++>-]>+++++++++++++++[<+++++++++++++++>-]<+>>++++++++++++[<+++++++++++++++++++>-]<->>++++++++++++[<+++++++++++++++++++>-]>++++++++++++[<+++++++++++++++++++>-]<+>>+++++++++++[<+++++++++++++++++++++>-]<->>+++++++++++[<+++++++++++++++++++++>-]>+++++++++++[<+++++++++++++++++++++>-]<+>>+++++++++++++[<++++++++++++++++++>-]<->>+++++++++++++[<++++++++++++++++++>-]>+++++++++++++[<++++++++++++++++++>-]<+>>++++++++++++++[<+++++++++++++++++>-]<-->>++++++++++++++[<+++++++++++++++++>-]<->>++++++++++++++[<+++++++++++++++++>-]>+++++++++++++++[<++++++++++++++++>-]<->>+++++++++++++++[<++++++++++++++++>-]>+++++++++++++++[<++++++++++++++++>-]<+>>+++++++++++++++[<++++++++++++++++>-]<++>>+++++++++++++++[<++++++++++++++++>-]<+++>>+++++++++++++[<+++++++++++++++++++>-]<--->>+++++++++++++[<+++++++++++++++++++>-]<-->>+++++++++++++[<+++++++++++++++++++>-]<->>+++++++++++++[<+++++++++++++++++++>-]>+++++++++++++[<+++++++++++++++++++>-]<+>>+++++++++++++[<+++++++++++++++++++>-]<++>>++++++++++++++[<++++++++++++++++++>-]<-->>++++++++++++++[<++++++++++++++++++>-]<->>++++++++++++++[<++++++++++++++++++>-]>++++++++++++++[<++++++++++++++++++>-]<+>>+++++++++++++++[<+++++++++++++++++>-]<->>+++++++++++++++[<+++++++++++++++++>-]");
    }

    #[test]
    fn test_print_bf_bytes() {
        //! Test the `print_bf_bytes` function.

        let text_bytes = text_2_bytes("Brainfuck").unwrap();
        let store_ord = [66, 97, 99, 102, 105, 107, 110, 114, 117];
        let mut data_ptr = 10;
        let bf_code = print_bf_bytes(&text_bytes, &store_ord, data_ptr);

        let mut out_str = String::new();
        for c in bf_code.chars() {
            match c {
                '>' => data_ptr += 1,
                '<' => data_ptr -= 1,
                '.' => out_str.push(store_ord[data_ptr] as char),
                _ => panic!("Unexpected Brainfuck command!"),
            }
        }
        assert_eq!(out_str, "Brainfuck");
    }

    #[test]
    fn test_factor_table() {
        //! Test the `factor_table` function.

        // since the only factors of prime numbers are 1 and the number itself,
        // the factor_table should not have difference 0 for prime numbers

        for (n, val) in factor_table().into_iter().enumerate() {
            if let Some((f1, f2, diff)) = val {
                assert_eq!(n as u8, (f1 * f2).checked_add_signed(diff).unwrap(), "The number {} is not represented correctly.", n);

                let (is_prime, _) = is_prime(n as u64);
                if is_prime {
                    assert_ne!(diff, 0, "The prime number {} has difference != 0.", n);
                }
            }
        }
    }
}
