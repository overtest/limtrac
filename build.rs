extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    /* @Creation of C and C++ library bindings with 'CBINDGEN' */
    cbindgen::generate(crate_dir)
        .expect("Unable to generate C bindings for LIMTRAC!")
        .write_to_file("bindings/limtrac.h");
    /* /@Creation of C and C++ library bindings with 'CBINDGEN' */
}