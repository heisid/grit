use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use flate2::read::ZlibDecoder;
use crate::die;

pub enum GitObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
    Undefined,
}

struct GitObject {
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

    fn from_file(path: PathBuf) -> Self {
        if !path.is_file() {
            return Self::new();
        }
        if let Ok(compressed_content) = File::open(path.clone()) {
            let sha = path.file_name().unwrap().to_str().unwrap();
            let decoder = ZlibDecoder::new(BufReader::new(compressed_content));
            let mut reader = ZlibDecoder::new(decoder);

            let mut type_buffer = Vec::new();
            let mut size_buffer = Vec::new();
            let mut data = Vec::new();
            let mut byte = [0u8; 1];

            let mut current_segment: u8 = 0; // 0: type, 1: size, 2: content
            while reader.read_exact(&mut byte).is_ok() {
                match current_segment {
                    0 => {
                        if byte[0] != b' ' {
                            type_buffer.push(byte[0]);
                        } else {
                            current_segment += 1;
                        }
                    }
                    1 => {
                        if byte[0] == b'\x00' {
                            size_buffer.push(byte[0]);
                        } else {
                            current_segment += 1;
                        }
                    }
                    _ => {
                        data.push(byte[0]);
                    }
                }
            }
            let object_type = match String::from_utf8_lossy(&type_buffer).to_string().as_str() {
                "commit" => { GitObjectType::Commit }
                "tree" => { GitObjectType::Tree }
                "tag" => { GitObjectType::Tag }
                "blob" => { GitObjectType::Blob }
                _ => {
                    die!(
                        "Unknown object type: {} in object {}",
                        String::from_utf8_lossy(&type_buffer),
                        sha
                    );
                }
            };
            let Ok(size) = String::from_utf8_lossy(&size_buffer).to_string().parse::<usize>() else {
                die!("Malformed object {}, unknown size", sha);
            };
            Self {
                object_type,
                size,
                data,
            }
        } else {
            Self::new()
        }
    }
}