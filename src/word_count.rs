use std::{cmp::Ordering, collections::HashMap, io::Write, sync::Mutex};

use caseless::Caseless;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::mt::MtDeque;

pub fn count_words(contents: &str, mt_d_indexes: &MtDeque<Option<HashMap<String, usize>>>) {
    let mut local_map = HashMap::<String, usize>::new();

    for word in contents.unicode_words() {
        let norm_word = compatibility_case_fold(word);

        *local_map.entry(norm_word).or_insert(0) += 1;
    }

    mt_d_indexes.push_back(Some(local_map));
}

pub fn compatibility_case_fold(s: &str) -> String {
    // TODO: This looks stupid
    s.nfd()
        .default_case_fold()
        .nfkd()
        .default_case_fold()
        .nfkd()
        .collect()
}

pub fn index_files_from_deque(
    mt_d_file_contents: &MtDeque<Option<String>>,
    // global_map: &Mutex<HashMap<String, usize>>,
    mt_d_indexes: &MtDeque<Option<HashMap<String, usize>>>
) {
    loop {
        let file_contents = match mt_d_file_contents.pop_front() {
            Some(v) => v,
            None => {
                mt_d_file_contents.push_back(None);
                break;
            }
        };

        count_words(&file_contents, mt_d_indexes);
    }

    mt_d_indexes.push_back(None);
}

fn write_sorted_map_to_file(
    global_map: &HashMap<String, usize>,
    path: &str,
    f: fn(&(&String, &usize), &(&String, &usize)) -> Ordering,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut sorted_map = global_map.into_iter().collect::<Vec<(&String, &usize)>>();
    sorted_map.sort_by(|word1, word2| f(word1, word2));

    // TODO: Better error handling
    let mut file = std::fs::File::create(path).unwrap();
    for (k, v) in sorted_map {
        writeln!(&mut file, "{} {}", k, v)?;
    }

    Ok(())
}

pub fn write_map_sorted_by_value(global_map: &HashMap<String, usize>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    write_sorted_map_to_file(global_map, path, |word1, word2| {
        if word1.1 != word2.1 {
            return word2.1.cmp(word1.1);
        } else {
            return word1.0.cmp(word2.0);
        }
    })?;

    Ok(())
}

pub fn write_map_sorted_by_key(global_map: &HashMap<String, usize>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    write_sorted_map_to_file(global_map, path, |word1, word2| {
        return word1.0.cmp(word2.0);
    })?;

    Ok(())
}
