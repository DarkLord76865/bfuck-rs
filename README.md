# bfuck-rs
**bfuck** is a simple Brainfuck interpreter and transpiler to Rust.

It is a CLI tool to interpret Brainfuck code,
to transpile Brainfuck code to (ugly) Rust code,
and to compile that Rust code.


### Installation
If you have both _rustc_ and _cargo_ installed, you can simply run:
```commandline
cargo install bfuck
```
Otherwise, there are prebuilt binaries available under Releases.

### Usage
```commandline
A simple Brainfuck interpreter and transpiler to Rust

Usage: bfuck [OPTIONS] <FILE> [FOLDER]

Arguments:
  <FILE>
          The brainfuck file
  [FOLDER]
          The save location for transpiled files

Options:
  -i, --interpret
          Interpret Brainfuck code [default]
  -t, --transpile
          Transpile Brainfuck code to Rust
  -c, --compile
          Transpile Brainfuck code to Rust and compile it (works only if Rust and Cargo are installed)
  -f, --force
          Overwrite output folder if it already exists
  -h, --help
          Print help
  -V, --version
          Print version
```
This information can be accessed by running:
```commandline
bfuck --help
```
