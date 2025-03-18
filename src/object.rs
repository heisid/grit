use std::fmt::{Display, Formatter};
use crate::utilities::{deserialize_kv_with_message, serialize_kv_with_message};
use crate::{create_path_or_die, die};
use derive_is_enum_variant::is_enum_variant;
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use indexmap::IndexMap;
use sha1::{Digest, Sha1};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

#[derive(is_enum_variant)]
pub enum GitObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
    Undefined,
}

pub struct GitObject {
    object_type: GitObjectType,
    size: usize,
    data: Vec<u8>,
}

impl GitObject {
    fn new() -> Self {
        Self {
            object_type: GitObjectType::Undefined,
            size: 0,
            data: Vec::new(),
        }
    }

    pub fn from_file(path: PathBuf) -> Self {
        if !path.is_file() {
            return Self::new();
        }
        if let Ok(compressed_content) = File::open(path.clone()) {
            let sha = path.file_name().unwrap().to_str().unwrap();
            let mut decoder = ZlibDecoder::new(compressed_content);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();

            let mut iter = decompressed.iter().peekable();

            let type_buffer: Vec<u8> = iter.by_ref()
                .take_while(|&&byte| byte != b' ')
                .cloned()
                .collect();

            let size_buffer: Vec<u8> = iter.by_ref()
                .take_while(|&&byte| byte != 0)
                .cloned()
                .collect();

            let data: Vec<u8> = iter.cloned().collect();

            let object_type = match String::from_utf8_lossy(&type_buffer).to_string().as_str() {
                "blob" => { GitObjectType::Blob }
                "commit" => { GitObjectType::Commit }
                "tag" => { GitObjectType::Tag }
                "tree" => { GitObjectType::Tree }
                _ => {
                    die!("Unknown type in object {}", sha);
                }
            };
            let Ok(size) = String::from_utf8_lossy(&size_buffer).to_string().parse::<usize>() else {
                die!("Malformed object {}, unknown size", sha);
            };
            if data.len() != size {
                die!("Malformed object {}, size doesn't match", sha);
            }
            Self {
                object_type,
                size,
                data,
            }
        } else {
            Self::new()
        }
    }

    pub fn write_to_file(&mut self, git_object_dir: PathBuf) {
        let object_type = match self.object_type {
            GitObjectType::Blob => { "blob " }
            GitObjectType::Commit => { "commit " }
            GitObjectType::Tag => { "tag " }
            GitObjectType::Tree => { "tree " }
            _ => { die!("Cannot write to object file, type unknown") }
        };
        let mut bytes_to_write = Vec::from(object_type);
        let mut size_in_bytes = Vec::from(self.size.to_string());
        bytes_to_write.append(&mut size_in_bytes);
        bytes_to_write.push(0);
        bytes_to_write.append(&mut self.data);

        let mut hasher = Sha1::new();
        hasher.update(&bytes_to_write);
        let hash = hasher.finalize();
        let sha = hash.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        let buff = BufReader::new(bytes_to_write.as_slice());
        let mut encoder = ZlibEncoder::new(buff, Compression::fast());
        let mut compressed = Vec::new();
        encoder.read_to_end(&mut compressed).unwrap();

        let object_dir = git_object_dir.join(&sha[..2]);
        create_path_or_die!(dir: object_dir.clone(), "Failed to create object file, could not create directory");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(object_dir.join(&sha[2..])).unwrap();
        file.write_all(compressed.as_slice()).unwrap();
    }
}

pub struct GitCommit {
    header: IndexMap<String, String>,
    message: String,
}

impl GitCommit {
    pub fn new() -> Self {
        Self {
            header: IndexMap::new(),
            message: String::new(),
        }
    }

    pub fn from_git_object(git_object: GitObject) -> Self {
        if !git_object.object_type.is_commit() {
            die!("Could not create commit object");
        }
        let (header, message) = deserialize_kv_with_message(&git_object.data);
        Self { header, message }
    }

    pub fn serialize(&self) -> Vec<u8> {
        serialize_kv_with_message(&self.header, self.message.as_str())
    }
}

impl Display for GitCommit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.header.clone() {
            write!(f, "{} {}\n", k, v)?;
        }
        write!(f, "\n")?;
        write!(f, "{}", self.message)?;
        Ok(())
    }
}

struct GitLeaf {
    mode: String,
    path: String,
    sha: String,
}

struct GitTree {
    records: Vec<GitLeaf>,
}

impl GitTree {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    fn parse_leaf(record: &[u8], offset: &mut usize) -> GitLeaf {
        // wacky
        let space_idx = record.iter().position(|&b| b == b' ');
        if space_idx.is_none() {
            die!("Malformed tree");
        }
        let mode_vec = record[..space_idx.unwrap()].to_vec();
        let mode = String::from_utf8(mode_vec);
        if mode.is_err() { die!("Malformed tree") }
        *offset += space_idx.unwrap();

        let record = &record[space_idx.unwrap() + 1..];
        let zero_byte_idx = record.iter().position(|&b| b == 0);
        if zero_byte_idx.is_none() {
            die!("Malformed tree");
        }
        let path_vec = record[..zero_byte_idx.unwrap()].to_vec();
        let path = String::from_utf8(path_vec);
        if path.is_err() { die!("Malformed tree") }
        *offset += zero_byte_idx.unwrap();

        let record = &record[zero_byte_idx.unwrap() + 1..];
        if record.len() < 20 {
            die!("Malformed tree");
        }
        *offset += 21;

        let sha = String::from_utf8(record.to_vec());
        if sha.is_err() { die!("Malformed tree") }

        GitLeaf {
            mode: mode.unwrap(),
            path: path.unwrap(),
            sha: sha.unwrap(),
        }
    }

    fn parse(data: &Vec<u8>) -> Self {
        let mut records: Vec<GitLeaf> = Vec::new();
        let mut offset: usize = 0;
        while offset < data.len() {
            let leaf = Self::parse_leaf(&data[offset..], &mut offset);
            records.push(leaf);
        }
        Self { records }
    }
}