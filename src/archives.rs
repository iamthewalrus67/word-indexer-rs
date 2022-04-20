pub fn unzip_from_memory(path: &str) {
    let file = std::fs::File::open(path).unwrap();
    let mut zip_archive = zip::ZipArchive::new(file).unwrap();
    let file_names = zip_archive
        .file_names()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    for file_name in &file_names {
        let zip_file = match zip_archive.by_name(file_name) {
            Ok(v) => v,
            Err(_) => continue,
        };

        println!("{:?}", zip_file.name());
    }
}
