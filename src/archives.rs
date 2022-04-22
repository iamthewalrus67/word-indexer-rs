use std::{io::{Read, Cursor}, path::{Path, PathBuf}, collections::VecDeque, rc::Rc, cell::RefCell};

use zip::{ZipArchive};

use crate::mt::{size_limits::FILE_SIZE_LIMIT_BYTES, MtDeque};
use crate::list_and_read::FileForIndex;

pub fn get_contents_from_zip_file(path: &str, zip_archive: RefCell<ZipArchive<Cursor<Vec<u8>>>>, mt_d_filenames: &MtDeque<Option<FileForIndex>>, mt_d_file_contents: &MtDeque<Option<String>>) {
    let mut zip_archive_borrow = zip_archive.borrow_mut();
    let mut zip_file =  zip_archive_borrow.by_name(path).unwrap();

    if zip_file.is_dir() {
        return;
    }

    if zip_file.size() >= FILE_SIZE_LIMIT_BYTES as u64{
        return;    
    }

    if zip_file.is_file() && Path::new(zip_file.name()).extension().unwrap() == "zip" {
        let mut buf = vec![];
        match zip_file.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(_) => return,
        }
        let zip_archive = RefCell::new(ZipArchive::new(Cursor::new(buf)).unwrap());
        get_file_names_from_zip_archive(zip_archive, mt_d_filenames);
        
    } else {
        let mut buf = vec![];
        match zip_file.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(_) => return,
        };

        mt_d_file_contents.push_back(Some(String::from_utf8_lossy(&buf).to_string()));
    }
}

pub fn get_file_names_from_zip_archive(zip_archive: RefCell<ZipArchive<Cursor<Vec<u8>>>>, mt_d_filenames: &MtDeque<Option<FileForIndex>>) {
    let file_names = zip_archive.borrow_mut().file_names().map(|s| s.to_string()).collect::<Vec<String>>();
    for file_name in file_names {
        if Path::new(&file_name).is_dir() {
            continue;
        }
        mt_d_filenames.push_back(Some(FileForIndex::Zip(file_name, RefCell::clone(&zip_archive))));
    }
}

pub fn get_file_names_from_zip_path(path: &Path, mt_d_filenames: &MtDeque<Option<FileForIndex>>) {
    let buf = std::fs::read(path).unwrap();
    let zip_archive = RefCell::new(ZipArchive::new(Cursor::new(buf)).unwrap());
    get_file_names_from_zip_archive(zip_archive, mt_d_filenames);
}

