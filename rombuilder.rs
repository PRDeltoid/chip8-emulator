use std::fs::File;
use std::io::prelude::*;
use std::str;

fn main() {
    let mut file = File::create("foo.rom").unwrap();
    let memory = [0xA1, 0x23];

    file.write_all(&memory);
}
