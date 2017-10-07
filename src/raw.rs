use byteorder_extended::{ReadExt, WriteExt};
use std::collections::HashMap;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};

/// Reads the pak header from the specified reader.
pub fn read_header<R: Read>(reader: &mut R) -> io::Result<()> {
    if reader.read_u8()? != 2 && reader.read_u8()? != 1 {
        Err(Error::new(
            ErrorKind::InvalidInput, 
            "Corrupted pak header"
        ))
    } else { Ok(()) }
}

/// Writes the pak header in the specified writer.
pub fn write_header<W: Write>(writer: &mut W) -> io::Result<()> {
    writer.write_u8(2)?;
    writer.write_u8(1)?;
    Ok(())
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

    /// Reads a new `Property` from the specified reader.
    pub fn from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let key = reader.read_string()?;
        let value = reader.read_string()?;
        Ok(Property::new(key, value))
    }

    /// Writes the `Property` in the specified writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_string(self.key.as_str())?;
        writer.write_string(self.value.as_str())?;
        Ok(())
    }

    /// Reads properties from the specified reader using offset and count `Info`.
    pub fn read<R: Read + Seek>(reader: &mut R, info: &Info) -> io::Result<HashMap<String, Self>> {
        let mut properties: HashMap<String, Property> = HashMap::with_capacity(info.properties_count as usize);

        reader.seek(SeekFrom::Start(info.properties_offset))?;
        for _ in 0..info.properties_count {
            let prop = Property::from(reader)?;
            properties.insert(prop.key.clone(), prop);
        }
        
        Ok(properties)
    }
}


/// Chunk
#[derive(Clone, Debug)]
pub struct Chunk {
    pub full_file_name: String,
    pub offset: i32,
    pub size: i32,
}

impl Chunk {
    /// Creates a new `Chunk`.
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

    /// Writes the `Chunk` in the specified writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_string(self.full_file_name.as_str())?;
        writer.write_i32(self.offset)?;
        writer.write_i32(self.size)?;
        Ok(())
    }

    /// Reads chunks from the specified reader using offset and count `Info`.
    pub fn read<R: Read + Seek>(reader: &mut R, info: &Info) -> io::Result<HashMap<String, Self>> {
        let mut chunks: HashMap<String, Chunk> = HashMap::with_capacity(info.chunks_count as usize);

        reader.seek(SeekFrom::Start(info.chunks_offset))?;
        for _ in 0..info.chunks_count {
            let chunk = Chunk::from(reader)?;
            chunks.insert(chunk.full_file_name.clone(), chunk);
        }

        Ok(chunks)
    }
}

/// Start position to read `Info` in a pak buffer.
static INFO_SEEK_ORIGIN: SeekFrom = SeekFrom::End(-24);

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
            properties_count
        }
    }

    /// Reads a new `Info` from the specified reader.
    pub fn from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        reader.seek(INFO_SEEK_ORIGIN)?;

        let offset = reader.read_i32()? as u64;
        let size = reader.read_i32()?;
        let chunks_offset = reader.read_i32()? as u64;
        let chunks_count =  reader.read_i32()?;
        let properties_offset = reader.read_i32()? as u64;
        let properties_count = reader.read_i32()?;

        Ok(Info::new(
            offset,
            size,
            chunks_offset,
            chunks_count,
            properties_offset,
            properties_count
        ))
    }

    /// Writes the `Info` in the specified writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i32(self.offset as i32)?;
        writer.write_i32(self.size)?;
        writer.write_i32(self.chunks_offset as i32)?;
        writer.write_i32(self.chunks_count)?;
        writer.write_i32(self.properties_offset as i32)?;
        writer.write_i32(self.properties_count)?;
        Ok(())
    }
}