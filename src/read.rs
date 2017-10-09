use raw::{Chunk, Info, Property, read_header};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Iter;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

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


/// MergedChunk
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

impl MergeReader<File> {
    pub fn open<P: AsRef<Path>>(loc: P) -> io::Result<Self> {
        MergeReader::merge(
            loc.as_ref(),
            |path| File::open(path)
        )
    }

    pub fn extract<P: AsRef<Path>>(loc: P, dest: P) -> io::Result<()> {
        let dest = dest.as_ref();
        let reader = MergeReader::<File>::open(&loc)?;

        for (full_file_name, chunk) in reader.iter() {
            // create the path
            let mut output = PathBuf::from(dest);
            output.push(full_file_name);

            // create the directory paths
            fs::create_dir_all(output.parent().unwrap())?;

            // create the file
            let mut file = File::create(&output)?;

            // fill the file with the data
            file.write_all(chunk.data().unwrap().as_slice())?;
        }

        Ok(())
    }
}

impl<R> MergeReader<R> 
where 
    R: Read + Seek
{
    fn merge<P, F>(initial: P, make_reader: F) -> io::Result<Self> 
        where P: Into<PathBuf>,
              F: Fn(PathBuf) -> io::Result<R>
    {
        let mut merge = MergeReader { 
            chunks: HashMap::new(),
            properties: HashMap::new()
        };

        let mut links = VecDeque::new();

        let initial = initial.into();
        links.push_back(initial.clone());

        while {
            let mut reader = make_reader(links.pop_front().unwrap())?;

            read_header(&mut reader)?;
            let info = Info::from(&mut reader)?;
            let mut chunks = Chunk::read(&mut reader, &info)?;
            let mut properties = Property::read(&mut reader, &info)?;

            let reader = Rc::new(RefCell::new(reader));

            for (full_file_name, chunk) in chunks.drain() {
                merge.chunks.insert(
                    full_file_name,
                    MergedChunk::new(
                        info.offset + chunk.offset as u64,
                        chunk.size as u64,
                        reader.clone()
                    )
                );
            }

            for (key, property) in properties.drain() {
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

    pub fn iter(&self) -> Iter<String, MergedChunk<R>> {
        self.chunks.iter()
    }
}