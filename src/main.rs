use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Arg, ArgAction, command, value_parser};

mod storage;
mod interpret;
mod transpile;
mod compile;

use storage::Storage;
use interpret::interpret;
use transpile::transpile;
use compile::compile;



fn main() {
    let argv = command!()
        .next_line_help(true)
        .arg(Arg::new("src_file")
            .value_name("FILE")
            .help("The brainfuck file")
            .required(true)
            .value_parser(value_parser!(PathBuf)))
        .arg(Arg::new("dst_folder")
            .value_name("FOLDER")
            .help("The save location for transpiled files")
            .required(false)
            .value_parser(value_parser!(PathBuf)))
        .arg(Arg::new("interpret")
            .short('i')
            .long("interpret")
            .action(ArgAction::SetTrue)
            .help("Interpret Brainfuck code [default]")
            .conflicts_with_all(["transpile", "compile"])
            .required(false))
        .arg(Arg::new("transpile")
            .short('t')
            .long("transpile")
            .action(ArgAction::SetTrue)
            .help("Transpile Brainfuck code to Rust")
            .requires("dst_folder")
            .conflicts_with_all(["interpret", "compile"])
            .required(false))
        .arg(Arg::new("compile")
            .short('c')
            .long("compile")
            .action(ArgAction::SetTrue)
            .help("Transpile Brainfuck code to Rust and compile it (works only if Rust and Cargo are installed)")
            .requires("dst_folder")
            .conflicts_with_all(["interpret", "transpile"])
            .required(false))
        .arg(Arg::new("force")
            .short('f')
            .long("force")
            .action(ArgAction::SetTrue)
            .help("Overwrite output folder if it already exists")
            .conflicts_with("interpret")
            .required(false))
        .get_matches();

    let src_file = Path::new(argv.get_one::<PathBuf>("src_file").unwrap().to_str().unwrap());
    let dst_folder = Path::new(match argv.get_one::<PathBuf>("dst_folder") {
        Some(path) => path.to_str().unwrap(),
        None => "",
    });
    let transpile_flag: bool = argv.get_flag("transpile");
    let compile_flag: bool = argv.get_flag("compile");
    let interpret_flag: bool = if transpile_flag || compile_flag {argv.get_flag("interpret")} else {true};
    let force_flag: bool = argv.get_flag("force");

    let brainfuck_code = match fs::read_to_string(src_file) {
        Ok(mut code) => {
            code.retain(|c| {
                c == '>' ||
                c == '<' ||
                c == '+' ||
                c == '-' ||
                c == '[' ||
                c == ']' ||
                c == '.' ||
                c == ','
            });
            code
        },
        Err(err) => {
            eprintln!("Error reading the Brainfuck file: {}", err);
            exit(1);
        },
    };

    if interpret_flag {
        interpret(brainfuck_code);
    } else if transpile_flag {
        transpile(brainfuck_code, src_file, dst_folder, force_flag);
    } else if compile_flag {
        compile(brainfuck_code, src_file, dst_folder, force_flag);
    }
}
