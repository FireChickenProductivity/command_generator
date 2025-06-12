use std::path::{PathBuf};
use std::fs;
use std::env::current_dir;
use crate::recommendation_generation::*;


fn create_directory_if_nonexistent(directory: &PathBuf) -> std::io::Result<()> {
	if !directory.exists() {
		fs::create_dir_all(&directory)?;
	}
	Ok(())
}

fn compute_data_directory() -> std::io::Result<PathBuf> {
	let mut data_directory = current_dir()?;
	data_directory.push("data");
	Ok(data_directory)
}

pub fn create_data_directory() -> std::io::Result<()> {
	let path = compute_data_directory()?;
	create_directory_if_nonexistent(&path)?;
	Ok(())
}



pub fn output_recommendations(
	recommendations: &[PotentialCommandInformation],
	file_name: &str,
) -> std::io::Result<()> {
	Ok(())
}
