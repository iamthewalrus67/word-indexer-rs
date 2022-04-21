mod archives;
mod list_and_read;
mod mt;
mod parsing;
mod word_count;
mod parallel_merge;

use std::env;
use std::sync::Arc;
use std::thread;

use mt::MtDeque;
use parallel_merge::merge;
use word_count::*;

use crate::list_and_read::{add_files_to_deque, read_files_from_deque};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let conf_path;
    if args.len() != 2 {
        conf_path = "index.cfg";
    } else {
        conf_path = &args[1];
    }

    let config = parsing::parse_config(conf_path)?;

    let mt_d_filenames = Arc::new(MtDeque::new());
    let mt_d_file_contents = Arc::new(MtDeque::new());
    let mt_d_indexes = Arc::new(MtDeque::new());

    let mut threads = vec![];

    // Create list thread
    {
        let mt_d_filenames = Arc::clone(&mt_d_filenames);
        let list_thread =
            thread::spawn(move || add_files_to_deque(&mt_d_filenames, &config.indir));
        threads.push(list_thread);
    }

    // Create read thread
    {
        let mt_d_filenames = Arc::clone(&mt_d_filenames);
        let mt_d_file_contents = Arc::clone(&mt_d_file_contents);
        let read_thread = thread::spawn(move || {
            read_files_from_deque(
                &mt_d_filenames,
                &mt_d_file_contents,
            )
        });
        threads.push(read_thread);
    }

    // Create index threads
    for _ in 0..config.indexing_threads {
        let mt_d_file_contents = Arc::clone(&mt_d_file_contents);
        let mt_d_indexes = Arc::clone(&mt_d_indexes);
        threads.push(thread::spawn(move || {
            index_files_from_deque(&mt_d_file_contents, &mt_d_indexes);
        }));
    }

    // Create merging threads
    let mut merging_threads = vec![];

    for _ in 0..config.merging_threads {
        let mt_d_indexes = Arc::clone(&mt_d_indexes);
        merging_threads.push(thread::spawn(move || {
            merge(&mt_d_indexes);
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    // Poison for hashmap deque
    mt_d_indexes.push_back(None);

    for thread in merging_threads {
        thread.join().unwrap();
    }

    let mut map = mt_d_indexes.pop_front().unwrap();
    if mt_d_indexes.size() > 1 {
        let mut kostyl_map = mt_d_indexes.pop_front().unwrap();
        parallel_merge::merge_into_first(&mut map, &mut kostyl_map)
    }

    write_map_sorted_by_key(&map, &config.out_by_a)?;
    write_map_sorted_by_value(&map, &config.out_by_n)?;

    Ok(())
}
