use std::fs;
use std::io::Read;
use std::path::PathBuf;

use crate::mt::{size_limits, MtDeque};

pub fn read_files_from_deque(
    mt_d_filenames: &MtDeque<Option<Box<PathBuf>>>,
    mt_d_file_contents: &MtDeque<Option<String>>,
) {
    loop {
        let file_path = match mt_d_filenames.pop_front() {
            Some(v) => v,
            None => {
                mt_d_filenames.push_front(None);
                break;
            }
        };

        if file_path.is_dir() {
            continue;
        }

        let file_ext: String = match file_path.extension() {
            Some(v) => match v.to_str() {
                Some(s) => s.to_string(),
                None => continue,
            },
            None => continue,
        };

        if file_ext == "zip" {
            crate::archives::unzip_from_memory(&file_path.as_os_str().to_str().unwrap(), mt_d_file_contents);
            continue;
        }

        let mut file = match fs::File::open(file_path.as_ref()) {
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
    mt_d_file_contents.push_front(None);
}

pub fn add_files_to_deque(mt_d_filenames: &MtDeque<Option<Box<PathBuf>>>, indir: &str) {
    use walkdir::WalkDir;

    for entry in WalkDir::new(indir).into_iter().filter_map(|e| e.ok()) {
        let path = PathBuf::from(entry.path().as_os_str().to_str().unwrap());
        mt_d_filenames.push_back(Some(Box::new(path)));
    }
    mt_d_filenames.push_front(None);
}
