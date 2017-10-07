extern crate pak;

use pak::*;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // new reader
    let reader = MergeReader::new(
        Path::new("D://Programmes/Ankama/Dofus_CW/app/content/gfx/monsters/monsters0.d2p"),
        |path| File::open(path)
    );

    if reader.is_err() {
        println!("{:?}", reader.err().unwrap());
        return;
    }

    let mut reader = reader.unwrap();
}