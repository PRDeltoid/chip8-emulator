use std::fs::File;
use std::io::prelude::*;
use std::str;

fn main() {
    let mut file = File::create("foo.rom").unwrap();
    let memory = [0x81, 0x26];

    file.write_all(&memory);
}
