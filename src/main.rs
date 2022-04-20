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
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 2 {
    //     eprintln!("Error: Wrong argument count");
    //     // TODO: Create enum of error codes
    //     exit(1);
    // }

    // let config = parsing::parse_config(&args[1])?;

    let config = parsing::parse_config("index.cfg")?;
    let mt_d_filenames = Arc::new(MtDeque::new());
    let mt_d_file_contents = Arc::new(MtDeque::new());

    let mt_d_filenames_arc_clone = Arc::clone(&mt_d_filenames);
    let mt_d_file_contents_arc_clone = Arc::clone(&mt_d_file_contents);

    let list_thread = thread::spawn(move || { add_files_to_deque(&mt_d_filenames_arc_clone, &config.indir) });

    let read_thread = thread::spawn(move || { read_files_from_deque(&mt_d_filenames, &mt_d_file_contents_arc_clone) });
    

    list_thread.join();
    read_thread.join();

    let mut global_map = Mutex::new(HashMap::<String, usize>::new());
    index_files_from_deque(&mt_d_file_contents, &mut global_map);

    write_map_sorted_by_key(&global_map.lock().unwrap(), &config.out_by_a);
    write_map_sorted_by_value(&global_map.lock().unwrap(), &config.out_by_n);

    Ok(())
}
