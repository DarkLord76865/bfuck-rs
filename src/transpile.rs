use std::fs;
use std::path::Path;
use std::process::exit;


pub fn transpile(brainfuck_code: String, src_file: &Path, dst_folder: &Path, force: bool) {
    create_dst_folder(dst_folder, force);

    let cargo_toml = generate_cargo_toml(src_file);
    let config_toml = generate_config_toml();
    let storage_rs = String::from(include_str!("storage.rs"));
    let mut main_rs = generate_main_rs();

    main_rs.push_str(&generate_rust_code(brainfuck_code));

    save_files(dst_folder, cargo_toml, config_toml, main_rs, storage_rs);
}


fn create_dst_folder(dst_folder: &Path, force: bool) {
    if dst_folder.exists() {
        if force {
            match fs::remove_dir_all(dst_folder) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("Error removing the existing destination folder: {}", err);
                    exit(1);
                }
            }
        } else {
            eprintln!("The specified destination folder already exists! Pass --force to overwrite it.");
            exit(1);
        }
    }
    match fs::create_dir_all(dst_folder) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error creating destination folder: {}", err);
            exit(1);
        }
    }
}

fn generate_cargo_toml(src_file: &Path) -> String {
    let mut cargo_toml = String::new();
    cargo_toml.push_str("[package]\n");
    cargo_toml.push_str(&format!("name = \"{}\"\n", src_file.file_stem().unwrap().to_str().unwrap()));
    cargo_toml.push_str("version = \"0.1.0\"\n");
    cargo_toml.push_str("edition = \"2021\"\n");
    cargo_toml.push('\n');
    cargo_toml.push_str("[dependencies]\n");
    cargo_toml
}

fn generate_config_toml() -> String {
    let major_platforms = [
        "aarch64-unknown-linux-gnu",
        "i686-pc-windows-gnu",
        "i686-pc-windows-msvc",
        "i686-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "x86_64-pc-windows-gnu",
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu",
    ];
    let mut config_toml = String::new();
    for (i, platform) in major_platforms.into_iter().enumerate()  {
        config_toml.push_str(&format!("[target.{}]\n", platform));
        config_toml.push_str("rustflags = [\"-C\", \"target-feature=+crt-static\"]\n");
        if i != major_platforms.len() - 1 {
            config_toml.push('\n');
        }
    }
    config_toml
}

fn generate_main_rs() -> String {
    let mut main_rs = String::new();
    main_rs.push_str("#![allow(unused_imports)]\n");
    main_rs.push('\n');
    main_rs.push_str("use std::io::{self, Read, Write};\n");
    main_rs.push_str("use std::ptr;\n");
    main_rs.push_str("use std::slice;\n");
    main_rs.push('\n');
    main_rs.push_str("mod storage;\n");
    main_rs.push_str("use storage::Storage;\n");
    main_rs.push('\n');
    main_rs.push('\n');
    main_rs.push_str("fn main() {\n");
    main_rs.push_str("    let mut storage = Storage::default();\n");
    main_rs.push_str("    let mut ptr: usize = 0;\n");
    main_rs.push_str("    #[allow(unused_variables)]\n");
    main_rs.push_str("    let stdin = io::stdin();\n");
    main_rs.push('\n');
    main_rs
}

fn generate_rust_code(raw_brainfuck: String) -> String {
    let parsed_brainfuck = parse_brainfuck(raw_brainfuck);
    let mut result = String::new();
    let mut indent: usize = 1;

    let add_indent = |string: &mut String, indent: usize| for _ in 0..(indent * 4) {string.push(' ')};

    for element in parsed_brainfuck {
        match element.0 {
            '+' => {
                add_indent(&mut result, indent);
                result.push_str(&format!("storage[ptr] = storage[ptr].wrapping_add(({} % 256_usize) as u8);\n", element.1));
            },
            '-' => {
                add_indent(&mut result, indent);
                result.push_str(&format!("storage[ptr] = storage[ptr].wrapping_add(((-{} % 256_isize) + 256_isize) as u8);\n", element.1));
            },
            '>' => {
                add_indent(&mut result, indent);
                result.push_str(&format!("ptr += {};\n", element.1));
            },
            '<' => {
                add_indent(&mut result, indent);
                result.push_str(&format!("if ptr < {} {{\n", element.1));
                indent += 1;
                add_indent(&mut result, indent);
                result.push_str("panic!(\"Data pointer index out of bounds!\");\n");
                indent -= 1;
                add_indent(&mut result, indent);
                result.push_str("} else {\n");
                indent += 1;
                add_indent(&mut result, indent);
                result.push_str(&format!("ptr -= {};\n", element.1));
                indent -= 1;
                add_indent(&mut result, indent);
                result.push_str("}\n");
            },
            '.' => {
                add_indent(&mut result, indent);
                if element.1 > 1 {
                    result.push_str(&format!("print!(\"{{}}\", format!(\"{{}}\", storage[ptr] as char).repeat({}));\n", element.1));
                } else {
                    result.push_str("print!(\"{}\", storage[ptr] as char);\n");
                }
            },
            ',' => {
                add_indent(&mut result, indent);
                result.push_str("io::stdout().flush().unwrap();");
                add_indent(&mut result, indent);
                if element.1 > 1 {
                    result.push_str(&format!("for _ in 0..{} {{\n", element.1));
                    indent += 1;
                    add_indent(&mut result, indent);
                    result.push_str("stdin.lock().read_exact(unsafe {slice::from_raw_parts_mut(ptr::addr_of_mut!(storage[ptr]), 1)}).expect(\"Error while reading input!\");\n");
                    indent -= 1;
                    add_indent(&mut result, indent);
                    result.push_str("}\n");
                } else {
                    result.push_str("stdin.lock().read_exact(unsafe {slice::from_raw_parts_mut(ptr::addr_of_mut!(storage[ptr]), 1)}).expect(\"Error while reading input!\");\n");
                }
            },
            '[' => {
                add_indent(&mut result, indent);
                result.push_str("while storage[ptr] != 0 {\n");
                indent += 1;
            },
            ']' => {
                indent -= 1;
                add_indent(&mut result, indent);
                result.push_str("}\n");
            },
            _ => panic!("Unknown command!"),
        }
    }
    add_indent(&mut result, indent);
    result.push_str("io::stdout().flush().unwrap();");
    result.push_str("}\n");

    result
}

fn parse_brainfuck(raw_brainfuck: String) -> Vec<(char, usize)> {
    let mut parsed_brainfuck: Vec<(char, usize)> = vec![(' ', 0)];
    for comm in raw_brainfuck.chars() {
        let last_element = parsed_brainfuck.len() - 1;
        if comm == '[' || comm == ']' {
            parsed_brainfuck.push((comm, 1));
        } else if parsed_brainfuck[last_element].0 == comm {
            parsed_brainfuck[last_element].1 += 1;
        } else {
            parsed_brainfuck.push((comm, 1));
        }
    }
    parsed_brainfuck.remove(0);
    parsed_brainfuck
}

fn save_files(dst_folder: &Path, cargo_toml: String, config_toml: String, main_rs: String, storage_rs: String) {
    match fs::create_dir_all(dst_folder.join(".cargo")) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error creating .cargo folder: {}", err);
            exit(1);
        }
    }
    match fs::create_dir_all(dst_folder.join("src")) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error creating src folder: {}", err);
            exit(1);
        }
    }

    match fs::write(dst_folder.join("Cargo.toml"), cargo_toml) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error writing Cargo.toml: {}", err);
            exit(1);
        }
    }
    match fs::write(dst_folder.join(".cargo").join("config.toml"), config_toml) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error writing .cargo/config.toml: {}", err);
            exit(1);
        }
    }
    match fs::write(dst_folder.join("src").join("main.rs"), main_rs) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error writing src/main.rs: {}", err);
            exit(1);
        }
    }
    match fs::write(dst_folder.join("src").join("storage.rs"), storage_rs) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Error writing src/storage.rs: {}", err);
            exit(1);
        }
    }
}
