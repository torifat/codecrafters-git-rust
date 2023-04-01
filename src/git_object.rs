use anyhow::{Context, Error, Result};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fmt::{Display, Formatter, Result as ForamtResult};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const OBJECTS_PATH: &str = ".git/objects";

#[derive(Debug)]
pub enum ObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> ForamtResult {
        match self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Commit => write!(f, "commit"),
            ObjectType::Tag => write!(f, "tag"),
            ObjectType::Tree => write!(f, "tree"),
        }
    }
}

#[derive(Debug)]
pub struct GitObject {
    pub object_type: ObjectType,
    pub content: Vec<u8>,
}

impl GitObject {
    pub fn new_from_object(hash: &str) -> Result<Self> {
        Result::<&str>::Ok(hash)
            .map(|hash| hash.split_at(2))
            .map(|(dir, file)| Path::new(OBJECTS_PATH).join(dir).join(file))
            .and_then(|file| {
                File::open(&file).with_context(|| format!("Failed to open file - {:?}", file))
            })
            .map(BufReader::new)
            .map(ZlibDecoder::new)
            .and_then(|mut decoder| {
                let mut data = Vec::new();
                decoder
                    .read_to_end(&mut data)
                    .map(|_| data)
                    .map_err(Error::from)
            })
            .map(Self::from)
    }

    pub fn new_from_file(file: &str) -> Result<Self> {
        File::open(&file)
            .with_context(|| format!("Failed to open file - {:?}", file))
            .map(BufReader::new)
            .and_then(|mut reader| {
                let mut data = Vec::new();
                reader
                    .read_to_end(&mut data)
                    .map(|_| data)
                    .map_err(Error::from)
            })
            .map(|content| Self {
                object_type: ObjectType::Blob,
                content,
            })
    }

    fn header(&self) -> String {
        format!("{} {}", self.object_type, self.content.len())
    }

    fn object(&self) -> Vec<u8> {
        [self.header().as_bytes(), &[0x00], self.content.as_slice()].concat()
    }

    pub fn hash(&self) -> String {
        Sha1::digest(self.object())
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn write(&self) -> Result<String> {
        let hash = self.hash();
        let (dir, file) = hash.split_at(2);
        let path = Path::new(OBJECTS_PATH).join(dir);

        create_dir_all(&path)
            .map(|_| path.join(file))
            .map_err(Error::from)
            .and_then(|file| {
                File::create(&file).with_context(|| format!("Failed to create file - {:?}", file))
            })
            .map(BufWriter::new)
            .and_then(|mut writer| {
                ZlibEncoder::new(&mut writer, Compression::default())
                    .write_all(self.object().as_slice())
                    .context("Failed to write to file")
            })?;

        Ok(hash)
    }
}

impl From<Vec<u8>> for GitObject {
    fn from(data: Vec<u8>) -> Self {
        let null_byte_position = data
            .iter()
            .position(|&byte| byte == b'\x00')
            .expect("Failed to find the null byte");
        let header = std::str::from_utf8(&data[..null_byte_position])
            .expect("Failed to convert header to UTF-8");
        let header_parts: Vec<&str> = header.split(' ').collect();

        assert_eq!(header_parts.len(), 2, "Header must have exactly two parts");

        let object_type = match header_parts[0] {
            "blob" => ObjectType::Blob,
            "commit" => ObjectType::Commit,
            "tag" => ObjectType::Tag,
            "tree" => ObjectType::Tree,
            _ => panic!("Invalid object type"),
        };

        let content = data[null_byte_position + 1..].to_vec();

        GitObject {
            object_type,
            content,
        }
    }
}
