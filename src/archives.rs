use std::{io::{Read, Cursor}, path::Path, collections::VecDeque};

use zip::ZipArchive;

use crate::mt::{size_limits::FILE_SIZE_LIMIT_BYTES, MtDeque};

pub fn unzip_from_memory(path: &str, mt_d_file_contents: &MtDeque<Option<String>>) {
    let file = std::fs::read(path).unwrap();
    let mut zip_archives_deque: VecDeque<ZipArchive<Cursor<Vec<u8>>>> = VecDeque::new();
    zip_archives_deque.push_back(ZipArchive::new(Cursor::new(file)).unwrap());
    
    loop {
        let mut zip_archive = match zip_archives_deque.pop_front() {
            Some(v) => v,
            None => break,
        };

        let file_names = zip_archive.file_names().map(|s| s.to_string()).collect::<Vec<String>>();
        for file_name in &file_names {
            let mut zip_file = match zip_archive.by_name(file_name) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if zip_file.is_dir() {
                continue;
            }

            if zip_file.is_file() && Path::new(zip_file.name()).extension().unwrap() == "zip" {
                let mut buf = vec![];
                zip_file.read_to_end(&mut buf);
                zip_archives_deque.push_back(ZipArchive::new(Cursor::new(buf)).unwrap());
                continue;
            }

            let mut buf = String::new();
            zip_file.read_to_string(&mut buf);
            mt_d_file_contents.push_back(Some(buf));
        }
    }
}
