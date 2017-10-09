extern crate pak;

use pak::*;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("the extractor needs 2 parameters : extract output_location pak_file_location");
        return 
    }

    MergeReader::extract(Path::new(&args[1]), Path::new(&args[2]));
}