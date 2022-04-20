use std::collections::HashMap;

use word_indexer_rs::mt::MtDeque;

pub fn merge(mt_d_indexes: MtDeque<Option<HashMap<String, usize>>>) {
    loop {
        let mut map1 = match mt_d_indexes.pop_front() {
            Some(v) => v,
            None => {
                mt_d_indexes.push_back(None);
                break;
            },
        };

        let mut map2 = mt_d_indexes.pop_front().unwrap();

        if map1.keys().len() < map2.keys().len() {
            merge_into_first(&mut map2, &mut map1);
            mt_d_indexes.push_back(Some(map2));
        } else {
            merge_into_first(&mut map1, &mut map2);
            mt_d_indexes.push_back(Some(map1));
        }
    }
}

fn merge_into_first(map1: &mut HashMap<String, usize>, map2: &mut HashMap<String, usize>) {
    for (k, v) in map2 {
        *map1.entry(k.to_string()).or_insert(0) += *v;
    }
}