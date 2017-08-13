//! # pak-rs
//!
//! **This crate provides a packaging tool.**
//!
//! _NB : Historically named Pak Protocol 2 with the file extensions pak,
//! pak2 or d2p. Since the Pak Protocol (d2pOld extension) is not used
//! anymore, Pak Protocol 2 becomes Pak Protocol._
//!
//! A pak file is an archive file without compression. The file extension 
//! is `d2p`. A pak file can be split in several files. A file segment
//! contains the path of the next segment to read.
//!
//! Pak file format :
//!
//! ``` text
//!     1. From Start 0
//!         header                  1 byte          value expected 2
//!         header                  1 byte          value expected 1
//! 
//!     2. From End -24
//!         offset_base             4 bytes         offset of any file
//!         unused                  4 bytes         _
//!         entries_offset          4 bytes         _
//!         entries_count           4 bytes         _
//!         properties_offset       4 bytes         _
//!         properties_count        4 bytes         _
//!
//!     3. From Start properties_offset
//!         for 0 to properties_count
//!             key                     2 bytes (length) | string (utf8)
//!             value                   2 bytes (length) | string (utf8)
//!         
//!             If the key equals to "link" then the value contains the path
//!             to the next segment of the pak file.
//!
//!     4. From Start entries_offset
//!         for 0 to entries_count
//!             file_name             2 bytes (length) | string (utf8)
//!             file_offset           4 bytes
//!             file_size             4 bytes
//! ```
//!
//! A file `file_name` can be load using the `offset_base` + `file_offset` 
//! with a size of `file_size` on the proper file segment.

extern crate byteorder_extended;

use std::fmt;
use std::fs::File;
use std::io::{ Error, Read };
use std::path::{ Path };

/// Loads a file and returns the bytes
fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    File::open(path)?.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Errors which can occur when attempting to interact with packaging tools.
#[derive(Clone, Debug, PartialEq)]
pub enum PakError {
    Unknown
}

impl fmt::Display for PakError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            //TODO: impl like this added errors :
            // PakError::MyError(ref err) => write!(f, "error : {}", err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PakFile;