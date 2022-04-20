use std::{collections::HashMap, sync::Mutex, cmp::Ordering, io::Write, fmt::format, fs::OpenOptions};

use unicode_segmentation::{UnicodeWords, UnicodeSegmentation};
use unicode_normalization::UnicodeNormalization;
use caseless::Caseless;

use crate::mt::MtDeque;

pub fn count_words(contents: &str, global_map: &Mutex<HashMap<String, usize>>) {
    let mut local_map = HashMap::<String, usize>::new();

    for word in contents.unicode_words() {
        let norm_word = compatibility_case_fold(word);

        *local_map.entry(norm_word).or_insert(0) += 1;
    }

    let mut guard = global_map.lock().unwrap();
    for (key, value) in local_map.into_iter() {
        *guard.entry(key).or_insert(0) += value;
    }
}

pub fn compatibility_case_fold(s: &str) -> String {
    // TODO: This looks stupid
    s.nfd().default_case_fold().nfkd().default_case_fold().nfkd().collect()
}

pub fn index_files_from_deque(mt_d_file_contents: &MtDeque<Option<(String, String)>>, global_map: &Mutex<HashMap<String, usize>>) {
    loop {
        let (file_contents, ext) = match mt_d_file_contents.pop_front() {
            Some(v) => v,
            None => {
                mt_d_file_contents.push_back(None);
                break;
            },
        };


        if ext == "txt" {
            count_words(&file_contents, global_map)
        }
    }
}

fn write_sorted_map_to_file(global_map: &HashMap<String, usize>,
                            path: &str,
                            f: fn(&(&String, &usize), &(&String, &usize)) -> Ordering) {
    let mut sorted_map = global_map.into_iter().collect::<Vec<(&String, &usize)>>();
    sorted_map.sort_by(|word1, word2| { f(word1, word2) });

    // TODO: Better error handling                            
    let mut file = std::fs::File::create(path).unwrap();
    for (k, v) in sorted_map {
        writeln!(&mut file, "{}: {}", k, v);
    }

}

pub fn write_map_sorted_by_value(global_map: &HashMap<String, usize>, path: &str) {
    write_sorted_map_to_file(global_map, path, |word1, word2| { 
        if (word1.1 != word2.1) {
            return word2.1.cmp(word1.1);
        } else {
            return word1.0.cmp(word2.0);
        }
     })
}

pub fn write_map_sorted_by_key(global_map: &HashMap<String, usize>, path: &str) {
    write_sorted_map_to_file(global_map, path, |word1, word2| { 
        return word1.0.cmp(word2.0);
     })
}