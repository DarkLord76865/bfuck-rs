use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Arg, ArgAction, command, value_parser};

use bfuck::interpret::interpret;
use bfuck::code::process_code;
use bfuck::jit::jit;
use bfuck::text::text_2_bf;

fn main() {
    let argv = command!()
        .next_line_help(true)
        .arg(Arg::new("src_file")
            .value_name("SRC_FILE")
            .help("The Brainfuck file.")
            .required(true)
            .value_parser(value_parser!(PathBuf))
        )
        .arg(Arg::new("interpret")
            .short('i')
            .long("interpret")
            .action(ArgAction::SetTrue)
            .help("Interpret Brainfuck code. [default]")
            .conflicts_with_all(["jit", "compile", "text_cvt"])
            .required(false)
        )
        .arg(Arg::new("jit")
            .short('j')
            .long("jit")
            .action(ArgAction::SetTrue)
            .help("Execute code using Just-in-time (JIT) compilation.")
            .conflicts_with_all(["interpret", "compile", "text_cvt"])
            .required(false)
        )
        .arg(Arg::new("compile")
            .short('c')
            .long("compile")
            .action(ArgAction::SetTrue)
            .help("Compile code to executable.")
            .conflicts_with_all(["interpret", "jit", "text_cvt"])
            .required(false)
        )
        .arg(Arg::new("dst_file")
            .value_name("DST_FILE")
            .help("The compiled file.")
            .requires("compile")
            .requires("text_cvt")
            .conflicts_with_all(["interpret", "jit"])
            .value_parser(value_parser!(PathBuf))
        )
        .arg(Arg::new("text_cvt")
            .short('t')
            .long("text_cvt")
            .action(ArgAction::SetTrue)
            .help("Converts the text file to Brainfuck code file which prints that text.")
            .conflicts_with_all(["interpret", "jit", "compile"])
            .required(false)
        )
        .get_matches();

    let src_file = Path::new(argv.get_one::<PathBuf>("src_file").unwrap().to_str().unwrap());
    let dst_file =
        match argv.get_one::<PathBuf>("dst_file") {
            Some(dst_file_path) => dst_file_path.as_path(),
            None => Path::new(""),
        };

    let mut interpret_flag: bool = argv.get_flag("interpret");
    let jit_flag: bool = argv.get_flag("jit");
    let compile_flag: bool = argv.get_flag("compile");
    let text_cvt_flag: bool = argv.get_flag("text_cvt");
    
    if !(interpret_flag || jit_flag || compile_flag || text_cvt_flag) {
        interpret_flag = true;
    }

    let src_text = match fs::read_to_string(src_file) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Error reading the file: {}", err);
            exit(1);
        },
    };

    if interpret_flag {
        let token_stream = match process_code(&src_text) {
            Ok(tokens) => tokens,
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            },
        };
        interpret(token_stream);
    } else if jit_flag {
        let token_stream = match process_code(&src_text) {
            Ok(tokens) => tokens,
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            },
        };
        if let Err(err) = jit(token_stream) {
            eprintln!("{}", err);
            exit(1);
        }
    } else if compile_flag {
        let _token_stream = match process_code(&src_text) {
            Ok(tokens) => tokens,
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            },
        };
    } else if text_cvt_flag {
        let bf_code = match text_2_bf(&src_text) {
            Ok(bf_code) => bf_code,
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            },
        };
        
        if let Err(err) = fs::write(dst_file, bf_code) {
            eprintln!("Error writing to the file: {}", err);
            exit(1);
        }
    }
}
