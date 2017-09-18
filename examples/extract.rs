extern crate pak;

use pak::*;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("the extractor needs 2 parameters : extract output_location pak_file_location");
        return 
    }

    let output = PathBuf::from(&args[1]);

    // new reader
    let reader = MergeReader::new(
        Path::new(&args[1]),
        |path| File::open(path)
    );

    if reader.is_err() {
        println!("{:?}", reader.err().unwrap());
        return;
    }

    let mut reader = reader.unwrap();

    for (full_file_name, mut chunk) in reader.iter() {
        // create the path
        let mut output = output.clone();
        output.push(full_file_name);

        // create the directory paths
        fs::create_dir_all(output.parent().unwrap());

        // create the file
        let mut file = File::create(&output).unwrap();

        // fill the file with the data
        file.write_all(chunk.data().unwrap().as_slice());
    }
}