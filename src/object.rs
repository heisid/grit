use crate::die;
use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

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

            let mut type_buffer = Vec::new();
            let mut size_buffer = Vec::new();
            let mut data = Vec::new();

            let mut current_segment: u8 = 0; // 0: type, 1: size, 2: content
            decompressed.iter().for_each(|&byte| {
                match current_segment {
                    0 => {
                        if byte != b' ' {
                            type_buffer.push(byte);
                        } else {
                            current_segment += 1;
                        }
                    }
                    1 => {
                        if byte != 0 {
                            size_buffer.push(byte);
                        } else {
                            current_segment += 1;
                        }
                    }
                    _ => {
                        data.push(byte);
                    }
                }
            });
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
            if data.len() != size {
                die!("Malformed object {}, size doesn't match", sha, size);
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
}