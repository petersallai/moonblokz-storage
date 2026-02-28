/* Embedded build helper for RP2040 example linker configuration. */

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap_or_default());
    let mut memory_x = File::create(out.join("memory.x")).unwrap_or_else(|_| {
        panic!("failed to create memory.x in OUT_DIR")
    });
    memory_x
        .write_all(include_bytes!("memory.x"))
        .unwrap_or_else(|_| panic!("failed to write memory.x"));
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}

