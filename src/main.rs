mod parsing;
mod mt;
mod list_and_read;
mod archives;
mod word_count;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::thread;
use std::{env, process::exit};
use std::sync::{Arc, Mutex};

use mt::MtDeque;
use word_count::*;
use zip::read::ZipFile;

use crate::list_and_read::{add_files_to_deque, read_files_from_deque};
use crate::word_count::compatibility_case_fold;

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
    let mut global_map = Arc::new(Mutex::new(HashMap::<String, usize>::new()));

    let mut threads = vec![];

    // Create list thread
    {
        let mt_d_filenames = Arc::clone(&mt_d_filenames);
        let list_thread = thread::spawn(move || { add_files_to_deque(&Arc::clone(&mt_d_filenames), &config.indir) });
        threads.push(list_thread);
    }

    // Create read thread
    {
        let mt_d_filenames = Arc::clone(&mt_d_filenames);
        let mt_d_file_contents = Arc::clone(&mt_d_file_contents);
        let read_thread = thread::spawn(move || { read_files_from_deque(&Arc::clone(&mt_d_filenames), &Arc::clone(&mt_d_file_contents)) });
        threads.push(read_thread);
    }

    // Create index threads
    {
        for _ in 0..config.indexing_threads {
            let mt_d_file_contents = Arc::clone(&mt_d_file_contents);
            let mut global_map = Arc::clone(&global_map);
            threads.push(thread::spawn(move || index_files_from_deque(&mt_d_file_contents, &mut global_map)));
        }
    }

    for thread in threads {
        thread.join();
    }

    write_map_sorted_by_key(&global_map.lock().unwrap(), &config.out_by_a);
    write_map_sorted_by_value(&global_map.lock().unwrap(), &config.out_by_n);

    Ok(())
}
