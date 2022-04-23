use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use crate::archives::get_contents_from_zip_file;
use crate::mt::{size_limits, MtDeque};

#[derive(Debug)]
pub enum FileForIndex {
    Zip(Vec<String>, zip::read::ZipArchive<Cursor<Vec<u8>>>),
    Regular(PathBuf),
}

pub fn read_files_from_deque(
    mt_d_filenames: &MtDeque<Option<FileForIndex>>,
    mt_d_file_contents: &MtDeque<Option<String>>,
) {
    loop {
        let file_for_index = match mt_d_filenames.pop_front() {
            Some(v) => v,
            // Poison pill for mt_d_filenames
            None => {
                mt_d_filenames.push_front(None);
                break;
            }
        };

        match file_for_index {
            FileForIndex::Regular(file_path) => {
                if file_path.is_dir() {
                    continue;
                }

                let mut file = match fs::File::open(file_path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Ok(file_metadata) = file.metadata() {
                    if file_metadata.len() >= size_limits::FILE_SIZE_LIMIT_BYTES as u64 {
                        continue;
                    }
                } else {
                    continue;
                }

                let mut file_contents = vec![];
                match file.read_to_end(&mut file_contents) {
                    Ok(_) => (),
                    Err(_) => continue,
                };
                let file_contents_string = String::from_utf8_lossy(&file_contents).to_string();

                if !file_contents_string.is_empty() {
                    mt_d_file_contents.push_front(Some(file_contents_string))
                }
            }
            FileForIndex::Zip(paths, zip_archive) => {
                get_contents_from_zip_file(paths, zip_archive, mt_d_file_contents);
            }
        }
    }
    mt_d_file_contents.push_front(None);
}

pub fn add_files_to_deque(mt_d_filenames: &MtDeque<Option<FileForIndex>>, indir: &str) {
    use walkdir::WalkDir;

    for entry in WalkDir::new(indir).into_iter().filter_map(|e| e.ok()) {
        let entry_ext = match entry.path().extension() {
            Some(ext) => ext,
            None => continue,
        };
        if entry_ext == "zip" {
            crate::archives::get_file_names_from_zip_path(entry.path(), mt_d_filenames);
        } else if entry_ext == "txt" {
            let path = PathBuf::from(entry.path().as_os_str().to_str().unwrap());
            mt_d_filenames.push_back(Some(FileForIndex::Regular(path)));
        }
    }
    mt_d_filenames.push_back(None);
}
