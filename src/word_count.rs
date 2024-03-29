use std::{cmp::Ordering, collections::HashMap, io::Write};

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

    mt_d_indexes.push_front(Some(local_map));
}

pub fn compatibility_case_fold(s: &str) -> String {
    // This looks stupid, but it is the correct way to do it
    s.nfd()
        .default_case_fold()
        .nfkd()
        .default_case_fold()
        .nfkd()
        .collect()
}

pub fn index_files_from_deque(
    mt_d_file_contents: &MtDeque<Option<String>>,
    mt_d_indexes: &MtDeque<Option<HashMap<String, usize>>>,
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
}

fn write_sorted_map_to_file(
    map: &HashMap<String, usize>,
    path: &str,
    f: fn(&(&String, &usize), &(&String, &usize)) -> Ordering,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut sorted_map = map.into_iter().collect::<Vec<(&String, &usize)>>();
    sorted_map.sort_by(|word1, word2| f(word1, word2));

    let mut file = std::fs::File::create(path).unwrap();
    for (k, v) in sorted_map {
        writeln!(&mut file, "{} {}", k, v)?;
    }

    Ok(())
}

pub fn write_map_sorted_by_value(
    map: &HashMap<String, usize>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_sorted_map_to_file(map, path, |word1, word2| {
        if word1.1 != word2.1 {
            return word2.1.cmp(word1.1);
        } else {
            return word1.0.cmp(word2.0);
        }
    })?;

    Ok(())
}

pub fn write_map_sorted_by_key(
    map: &HashMap<String, usize>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_sorted_map_to_file(map, path, |word1, word2| {
        return word1.0.cmp(word2.0);
    })?;

    Ok(())
}
