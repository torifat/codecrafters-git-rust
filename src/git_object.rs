use anyhow::{Context, Error, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fmt::{Display, Formatter, Result as ForamtResult};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::str::from_utf8;

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

impl From<&str> for ObjectType {
    fn from(s: &str) -> Self {
        match s {
            "blob" => ObjectType::Blob,
            "commit" => ObjectType::Commit,
            "tag" => ObjectType::Tag,
            "tree" => ObjectType::Tree,
            _ => panic!("Invalid object type"),
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

    // [mode] [file/folder name]\0[SHA-1 of referencing blob or tree]
    fn object(&self) -> Vec<u8> {
        [self.header().as_bytes(), &[0x00], self.content.as_slice()].concat()
    }

    pub fn hash(&self) -> String {
        hex::encode(Sha1::digest(self.object()))
    }

    pub fn print(&self) -> Result<()> {
        // [mode] [file/folder name]\0[SHA-1 of referencing blob or tree]
        match self.object_type {
            ObjectType::Blob => print!("{}", from_utf8(self.content.as_slice())?),
            ObjectType::Tree => {
                let mut it = self.content.clone().into_iter().peekable();

                while let Some(byte) = it.peek() {
                    if *byte != b' ' {
                        it.next();
                    } else {
                        // Skip the space character
                        it.next();
                        let tmp: Vec<u8> =
                            it.by_ref().take_while(|&byte| byte != b'\x00').collect();
                        println!("{}", from_utf8(tmp.as_slice())?);

                        // Skip the SHA-1
                        for _ in 0..20 {
                            it.next();
                        }
                    }
                }
            }
            _ => unreachable!(),
        };

        Ok(())
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
                ZlibEncoder::new(&mut writer, Compression::fast())
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
        let header =
            from_utf8(&data[..null_byte_position]).expect("Failed to convert header to UTF-8");
        let header_parts: Vec<&str> = header.split(' ').collect();

        assert_eq!(header_parts.len(), 2, "Header must have exactly two parts");

        let object_type = header_parts[0].into();
        let content = data[null_byte_position + 1..].to_vec();

        GitObject {
            object_type,
            content,
        }
    }
}
