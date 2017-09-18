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

use byteorder_extended::{ReadExt, WriteExt};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::iter::Map;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const INFO_OFFSET: i64 = -24;
static INFO_SEEK_ORIGIN: SeekFrom = SeekFrom::End(INFO_OFFSET);

/// Loads a file and returns the bytes.
pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    File::open(path)?.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Replaces the file name (with extension) of the `path` to `file_name`.
fn set_file_name<P: AsRef<Path>>(path: P, file_name: &str) -> Option<PathBuf> {
    path.as_ref()
        .to_str()
        .and_then(|path| {
            path.rfind('/').and_then(|last_index| {
                Some(PathBuf::from(
                    &[ 
                        &path[..last_index], 
                        "/", 
                        file_name
                    ].concat()
                ))
            }).or(Some(PathBuf::from(file_name)))
        })
}

fn read_header<R: Read>(reader: &mut R) -> io::Result<()> {
    if reader.read_u8()? != 2 && reader.read_u8()? != 1 {
        Err(Error::new(
            ErrorKind::InvalidInput, 
            "Corrupted pak header"
        ))
    } else { Ok(()) }
}

/// Property
#[derive(Clone, Debug)]
pub struct Property {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
}

impl Property {
    /// Creates a new `Property`.
    pub fn new(key: String, value: String) -> Self {
        Property {
            key,
            value
        }
    }

    /// Reads a new property from the specified reader.
    pub fn from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let key = reader.read_string()?;
        let value = reader.read_string()?;
        Ok(Property::new(key, value))
    }
}


/// Chunk
#[derive(Clone, Debug)]
pub struct Chunk {
    full_file_name: String,
    offset: i32,
    size: i32,
}

impl Chunk {
    pub fn new(full_file_name: String, offset: i32, size: i32) -> Self {
        Chunk {
            full_file_name,
            offset,
            size
        }
    }

    /// Creates a new `Chunk` from the specified reader.
    pub fn from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let full_file_name = reader.read_string()?;
        let offset = reader.read_i32()?;
        let size = reader.read_i32()?;
        Ok(Chunk::new(full_file_name, offset, size))
    }
}

/// Info
#[derive(Clone, Debug)]
pub struct Info {
    /// Offset base when attempting to load data 
    pub offset: u64,
    /// Size
    pub size: i32,
    /// Offset to start reading chunks
    pub chunks_offset: u64,
    /// Number of chunks
    pub chunks_count: i32,
    /// Offset to start reading properties
    pub properties_offset: u64,
    /// Number of properties
    pub properties_count: i32,

    /// Chunks.
    pub(crate) chunks: HashMap<String, Chunk>,
    /// Properties.
    pub(crate) properties: HashMap<String, Property>,
}

impl Info {
    /// Creates a new `Info`.  
    fn new(
        offset: u64,
        size: i32,
        chunks_offset: u64,
        chunks_count: i32,
        properties_offset: u64,
        properties_count: i32,
        ) -> Self {
        Info {
            offset,
            size,
            chunks_offset,
            chunks_count,
            properties_offset,
            properties_count,
            chunks: HashMap::new(),
            properties: HashMap::new()
        }
    }

    /// Reads a new `Info` from the specified reader.
    pub fn from<R: Read + Seek>(mut reader: &mut R) -> io::Result<Self> {
        reader.seek(INFO_SEEK_ORIGIN)?;

        let offset = reader.read_i32()? as u64;
        let size = reader.read_i32()?;
        let chunks_offset = reader.read_i32()? as u64;
        let chunks_count =  reader.read_i32()?;
        let properties_offset = reader.read_i32()? as u64;
        let properties_count = reader.read_i32()?;

        let mut info = Info::new(
            offset,
            size,
            chunks_offset,
            chunks_count,
            properties_offset,
            properties_count
        );

        reader.seek(SeekFrom::Start(info.properties_offset))?;
        for _ in 0..info.properties_count {
            let prop = Property::from(&mut reader)?;
            info.properties.insert(prop.key.clone(), prop);
        }

        reader.seek(SeekFrom::Start(info.chunks_offset))?;
        for _ in 0..info.chunks_count {
            let chunk = Chunk::from(&mut reader)?;
            info.chunks.insert(chunk.full_file_name.clone(), chunk);
        }

        Ok(info)
    }
}

///
#[derive(Debug)]
pub struct MergedChunk<R> {
    offset: u64, 
    size: u64,
    reader: Rc<RefCell<R>>,
}

impl<R> MergedChunk<R>
where 
    R: Read + Seek
{
    /// Creates a new `MergedChunk`.
    fn new(
        offset: u64,
        size: u64,
        reader: Rc<RefCell<R>>
    ) -> Self {
        MergedChunk {
            offset: offset,
            size: size,
            reader: reader
        }
    }

    /// Reads the data.
    pub fn data(&self) -> io::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![0; self.size as usize];
        {
            let mut reader = self.reader.borrow_mut();
            reader.seek(SeekFrom::Start(self.offset))?;
            reader.read(&mut buffer)?;
        }
        Ok(buffer)
    }
}

/// MergeReader
///
/// `MergeReader` merges the fragments and provides only functions
/// to read files from chunks of the whole archive.
///
/// Use `MergeReader` for a fast data reading.
#[derive(Debug)]
pub struct MergeReader<R> {
    /// Chunks 
    chunks: HashMap<String, MergedChunk<R>>,
    /// Properties
    properties: HashMap<String, String>,
}

impl<R> MergeReader<R> 
where 
    R: Read + Seek
{
    pub fn new<P, F>(initial: P, make_reader: F) -> io::Result<Self> 
        where P: Into<PathBuf>,
              F: Fn(PathBuf) -> io::Result<R>
    {
        let mut merge = MergeReader { 
            chunks: HashMap::new(),
            properties: HashMap::new()
        };

        let initial = initial.into();

        let mut links = VecDeque::new();
        links.push_back(initial.clone());

        while {
            let mut reader = make_reader(links.pop_front().unwrap())?;

            read_header(&mut reader)?;
            let mut info = Info::from(&mut reader)?;

            let reader = Rc::new(RefCell::new(reader));

            for (full_file_name, chunk) in info.chunks.drain() {
                merge.chunks.insert(
                    full_file_name,
                    MergedChunk::new(
                        info.offset + chunk.offset as u64,
                        chunk.size as u64,
                        reader.clone()
                    )
                );
            }

            for (key, property) in info.properties.drain() {
                if key.eq("link") {
                    links.push_back(set_file_name(&initial, &property.value).unwrap());
                }
                merge.properties.insert(property.key, property.value);
            }

            !links.is_empty()
        } {}

        Ok(merge)
    }

    pub fn read_file(&mut self, full_file_name: &str) -> io::Result<Vec<u8>> {
         self.chunks.get(full_file_name).map(|chunk| chunk.data()).unwrap_or(
            Err(Error::new(
                ErrorKind::InvalidInput, 
                format!("`full_file_name` \"{}\" can't be read", full_file_name)
            ))
        )
    }

    pub fn iter(&mut self) -> HashMap<&String, &MergedChunk<R>> {
        self.chunks.iter().map(|(full_file_name, chunk)|  {
            (full_file_name, chunk)
        }).collect()
    }
}