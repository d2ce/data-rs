//! # pak-rs
//!
//! **This crate provides a packaging tool.**
//!
//! _NB : Historically named Pak Protocol 2 with the file extensions `pak`, `pak2`
//! or `d2p`. Since the Pak Protocol (`d2pOld` extension) is not used anymore, 
//! Pak Protocol 2 becomes Pak Protocol._
//!
//! A pak file is an archive file without compression. The file extension is `d2p`.
//! A pak file can be split in several files. A file segment contains the path of
//! the next segment to read.
//!
//! Pak file format :
//!
//! ``` text
//!     1. Header : From Start 0
//!         +----------------------+----------------+-----------------------+
//!         |   header             |    1 byte      |   value expected 2    |
//!         |   header             |    1 byte      |   value expected 1    |
//!         +----------------------+----------------+-----------------------+
//! 
//!     2. Info : From End -24
//!         +----------------------+----------------+-----------------------+
//!         |   offset             |    4 bytes     |   offset of chunk data|
//!         |   size               |    4 bytes     |   _                   |
//!         |   chunks_offset      |    4 bytes     |   _                   |
//!         |   chunks_count       |    4 bytes     |   _                   |
//!         |   properties_offset  |    4 bytes     |   _                   |
//!         |   properties_count   |    4 bytes     |   _                   |
//!         +----------------------+----------------+-----------------------+
//!
//!     3. Properties : From Start properties_offset
//!         for 0 to properties_count
//!             +---------------------+------------------------------------+
//!             |   key               |  2 bytes (length) | string (utf8)  |
//!             |   value             |  2 bytes (length) | string (utf8)  |
//!             +---------------------+------------------------------------+
//!
//!             If the key equals to "link" then the value contains the relative
//!             path to the next fragment of the pak file.
//!
//!     4. Chunks : From Start chunks_offset
//!         for 0 to chunks_count
//!             +---------------------+------------------------------------+
//!             |   name              |   2 bytes (length) | string (utf8) |
//!             |   offset            |   4 bytes                          |
//!             |   size              |   4 bytes                          |
//!             +---------------------+------------------------------------+
//! ```
//!
//! The data described by a chunk can be load starting from the `Info.offset` + 
//! `Chunk.offset`.

extern crate byteorder_extended;

pub mod raw;

mod read;
mod write;

pub use read::{MergedChunk, MergeReader};