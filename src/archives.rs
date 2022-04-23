use std::{
    collections::VecDeque,
    ffi::OsStr,
    io::{Cursor, Read},
    path::Path,
};

use zip::ZipArchive;

use crate::list_and_read::FileForIndex;
use crate::mt::{size_limits::FILE_SIZE_LIMIT_BYTES, MtDeque};

pub fn get_contents_from_zip_file(
    paths: Vec<String>,
    mut zip_archive: ZipArchive<Cursor<Vec<u8>>>,
    mt_d_file_contents: &MtDeque<Option<String>>,
) {
    for path in paths {
        let mut zip_file = match zip_archive.by_name(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if zip_file.size() >= FILE_SIZE_LIMIT_BYTES as u64 {
            continue;
        }

        let mut buf = vec![];
        match zip_file.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(_) => continue,
        };
        mt_d_file_contents.push_back(Some(String::from_utf8_lossy(&buf).to_string()));
    }
}

pub fn get_file_names_from_zip_archive(
    zip_archive: ZipArchive<Cursor<Vec<u8>>>,
    mt_d_filenames: &MtDeque<Option<FileForIndex>>,
) {
    let mut zip_archive_deque = VecDeque::new();
    zip_archive_deque.push_back(zip_archive);

    loop {
        let mut popped_zip_archive = match zip_archive_deque.pop_front() {
            Some(v) => v,
            None => break,
        };

        let regular_file_names = popped_zip_archive
            .file_names()
            .filter(|s| {
                let p = Path::new(s);
                "txt"
                    == match p.extension() {
                        Some(v) => v,
                        None => OsStr::new(""),
                    }
            })
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let zip_archive_file_names = popped_zip_archive
            .file_names()
            .filter(|s| {
                let p = Path::new(s);
                return "zip"
                    == match p.extension() {
                        Some(v) => v,
                        None => OsStr::new(""),
                    };
            })
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        for file_name in zip_archive_file_names {
            // let mut popped_zip_archive_borrow = popped_zip_archive.borrow_mut();
            let mut zip_file = match popped_zip_archive.by_name(&file_name) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut buf = vec![];
            match zip_file.read_to_end(&mut buf) {
                Ok(_) => (),
                Err(_) => continue,
            };

            let nested_zip_archive = ZipArchive::new(Cursor::new(buf)).unwrap();
            zip_archive_deque.push_back(nested_zip_archive);
        }

        mt_d_filenames.push_back(Some(FileForIndex::Zip(
            regular_file_names,
            popped_zip_archive,
        )));
    }
}

pub fn get_file_names_from_zip_path(path: &Path, mt_d_filenames: &MtDeque<Option<FileForIndex>>) {
    let buf = std::fs::read(path).unwrap();
    let zip_archive = match ZipArchive::new(Cursor::new(buf)) {
        Ok(v) => v,
        Err(_) => return,
    };
    get_file_names_from_zip_archive(zip_archive, mt_d_filenames);
}
