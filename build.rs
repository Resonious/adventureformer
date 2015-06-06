use std::env;

fn main() {
    println!("cargo:rustc-link-search=native=./clib");
    println!("cargo:libdir=./clib");
    println!("cargo:include=./cinclude");
}
