use std::fs::File;
use std::io::prelude::*;
use std::str;

fn main() {
    let file = File::create("foo.rom").unwrap();
    let memory = [0x12, 0x34];

    file.write_all(&memory);
}
