use image::open;
use std::{fs, io, path::{Path, PathBuf}, thread};
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::thread::JoinHandle;
use std::time::Duration;
use indicatif::ProgressBar;


/** Compression functions **/
fn compress_img(directory: &Path, file_name: &Path) {
    let width = 500;
    let height = 300;

    let mut file_path: PathBuf = PathBuf::new();
    file_path.push(directory);
    file_path.push(file_name);

    let img = open(&file_path);
    let result = img.expect("File could not be opened");

    // Build the image output path
    let mut image_output_path = PathBuf::new();
    image_output_path.push(&directory);
    image_output_path.push("compressed");
    image_output_path.push(&file_name);

    let thumbnail_image =
        result.resize_to_fill(width, height, image::imageops::FilterType::Lanczos3);

    thumbnail_image
        .save(&image_output_path)
        .expect("Failed to save image");
}

fn compress_directory(directory: &str) -> io::Result<()> {
    let mut entries = fs::read_dir(directory)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();

    let mut compressed_entries = fs::read_dir(format!("{}/{}",directory,"compressed")).unwrap().map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    compressed_entries.sort();

    let compressed_set: HashSet<OsString> = compressed_entries
        .into_iter()
        .map(|p| p.file_name().unwrap().to_os_string())
        .collect();


    let entries_to_move = entries.clone();
    let directory_path = PathBuf::from(directory);
    let bar = ProgressBar::new(entries.len() as u64);
    let mut thread_handles : Vec<JoinHandle<()>>  = Vec::new();

    for entry in entries_to_move.iter() {
        if entry.is_dir() || compressed_set.contains(&entry.file_name().unwrap().to_os_string()) {
            //println!("compression:{:?} : file  {:?} already present skipping file", &entry.file_name(), entry);
            continue;
        };

        let thread_dir = directory_path.clone();
        let file_name = entry.file_name().unwrap().to_os_string();
        let thread_bar = bar.clone();

        let thread = thread::spawn(move || {
            compress_img(
                &thread_dir,
                Path::new(&file_name)
            );
            thread_bar.inc(1);
        });

        thread_handles.push(thread);
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
    bar.finish_and_clear();
    Ok(())
}


/** Helpers **/
pub fn convert_path_to_compressed(path:PathBuf) -> PathBuf{
    let mut local = path.clone();
    let filename =  path.file_name();

    if let  Some(file) = filename {
        local.pop();
        local.push("compressed");
        local.push(file);
    }

    local
}

pub fn compress(base_directory: &str) -> PathBuf {
    // compress images if no images compressed
    let compressed_dir = std::path::Path::new(&base_directory).join("compressed");

    if !compressed_dir.exists(){
        // Create compressed folder if needed
        fs::create_dir(&compressed_dir).unwrap_or_default();

    }

    compress_directory(&base_directory).expect("Error: the directory could not be compressed.");

    compressed_dir
}
