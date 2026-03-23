use image::open;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn compress_img(directory: &Path, file_name: &Path) {
    let width = 500;
    let height = 300;

    let mut file_path: PathBuf = PathBuf::new();
    file_path.push(directory);
    file_path.push(file_name);

    let img = open(&file_path);
    let result = img.expect("File could not be opened");

    // Create compressed folder if needed

    fs::create_dir(directory.join("compressed")).unwrap_or_default();

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

pub fn compress_directory(directory: &str) -> io::Result<()> {
    let mut entries = fs::read_dir(directory)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();

    for entry in entries.iter() {
        if entry.is_dir() {
            continue;
        };

        compress_img(
            Path::new(directory),
            Path::new(entry.file_name().expect("huh")),
        );
    }

    Ok(())
}
