use std::collections::HashMap;
use std::fs;
use crate::{create_path_or_die, die};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use derive_is_enum_variant::is_enum_variant;
use flate2::Compression;
use sha1::{Digest, Sha1};
use crate::utilities::deserialize_kv_with_message;

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
    header: HashMap<String, String>,
    message: String,
}

impl GitCommit {
    pub fn new() -> Self {
        Self {
            header: HashMap::new(),
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
}