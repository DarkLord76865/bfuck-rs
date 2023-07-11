use std::path::Path;
use std::process::{Command, exit, Stdio};
use super::transpile;


pub fn compile(brainfuck_code: String, src_file: &Path, dst_folder: &Path, force: bool) {
    transpile(brainfuck_code, src_file, dst_folder, force);

    let check_cargo = Command::new("cargo")
        .arg("--version") // get cargo version
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status() // run and save exit status
        .expect("Error running Cargo.");

    if !check_cargo.success() {
        eprintln!("Error running Cargo.");
        exit(1);
    }

    let cargo_build = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(dst_folder)
        .status()
        .expect("Error compiling with Cargo.");

    if !cargo_build.success() {
        eprintln!("Error compiling with Cargo.");
        exit(1);
    }
}
