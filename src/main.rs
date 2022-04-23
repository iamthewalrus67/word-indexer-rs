mod archives;
mod list_and_read;
mod mt;
mod parsing;
mod word_count;
mod parallel_merge;

use std::{env, sync::Mutex};
use std::sync::Arc;
use std::thread;
use std::time::{Instant, Duration};

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

    let whole_instant = Instant::now();
    let filenames_duration = Arc::new(Mutex::new(Duration::new(0, 0)));
    let file_contents_duration = Arc::new(Mutex::new(Duration::new(0, 0)));

    let mt_d_filenames = Arc::new(MtDeque::new());
    let mt_d_file_contents = Arc::new(MtDeque::new());
    let mt_d_indexes = Arc::new(MtDeque::new());

    let mut threads = vec![];

    // Create list thread
    {
        let mt_d_filenames = Arc::clone(&mt_d_filenames);
        let filenames_duration = Arc::clone(&filenames_duration);
        let list_thread =
            thread::spawn(move || { 
                let instant = Instant::now();
                add_files_to_deque(&mt_d_filenames, &config.indir);
                let mut file_contents_duration_guard = filenames_duration.lock().unwrap();
                *file_contents_duration_guard += instant.elapsed();
                
            });
        threads.push(list_thread);
    }

    // Create read thread
    {
        let mt_d_filenames = Arc::clone(&mt_d_filenames);
        let mt_d_file_contents = Arc::clone(&mt_d_file_contents);
        let file_contents_duration = Arc::clone(&file_contents_duration);
        let read_thread = thread::spawn(move || {
            let instant = Instant::now();
            read_files_from_deque(
                &mt_d_filenames,
                &mt_d_file_contents,
            );
            let mut file_contents_duration_guard = file_contents_duration.lock().unwrap();
            *file_contents_duration_guard += instant.elapsed();
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
    while mt_d_indexes.len() > 1 {
        let kostyl_map = mt_d_indexes.pop_front().unwrap();
        parallel_merge::merge_into_first(&mut map, &kostyl_map)
    }

    let whole_duration = whole_instant.elapsed();

    let write_instant = Instant::now();
    write_map_sorted_by_key(&map, &config.out_by_a)?;
    write_map_sorted_by_value(&map, &config.out_by_n)?;
    let write_duration = write_instant.elapsed();

    println!("Total={}", whole_duration.as_millis());
    println!("Reading={}", file_contents_duration.lock().unwrap().as_millis());
    println!("Finding={}", filenames_duration.lock().unwrap().as_millis());
    println!("Writing={}", write_duration.as_millis());

    Ok(())
}
