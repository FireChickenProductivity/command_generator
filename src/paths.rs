use std::env::current_dir;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn create_directory_if_nonexistent(directory: &PathBuf) -> io::Result<()> {
    if !directory.exists() {
        fs::create_dir_all(&directory)?;
    }
    Ok(())
}

pub fn compute_directory_under_current_directory(name: &str) -> io::Result<PathBuf> {
    let mut directory = current_dir()?;
    directory.push(name);
    Ok(directory)
}

pub fn create_directory_under_current_directory(name: &str) -> io::Result<PathBuf> {
    let directory = compute_directory_under_current_directory(name)?;
    create_directory_if_nonexistent(&directory)?;
    Ok(directory)
}

pub fn warn_about_nonexistent_file(name: &str) {
    println!("WARNING: The {} file does not exist.", name);
}

pub fn create_file(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        fs::File::create(path)?;
    }
    Ok(())
}
