# duplicate-finder
Duplicate file finder, written in Rust

Simple command-line utility that detects files with identical content within a directory tree. At present, no output is produced, but the intention is that CSV and JSON output will be produced.

The tool has been written as much as a Rust learning experience as to be useful. Notably, the `-p` option was added to gain experience with concurrecncy in Rust through the [Rayon](https://github.com/nikomatsakis/rayon) library. In order to build, use a recent nightly Rust (I'm using `1.22.0-nightly (01c65cb15 2017-09-20)`) and `cargo build`.

## Usage
````
duplicate-finder 0.1.0
Stuart Dootson <stuart.dootson@gmail.com>
Searchs for duplicated files in directory tree(s).

USAGE:
    duplicate-finder.exe [FLAGS] [OPTIONS] <directory>...

FLAGS:
    -h, --help        Prints help information
    -p, --parallel    Use multi-threaded finder
    -V, --version     Prints version information

OPTIONS:
    -t, --output-type <output_type>    Specify output format (CSV, JSON) [default: CSV]

ARGS:
    <directory>...    Directory tree root(s)
````
