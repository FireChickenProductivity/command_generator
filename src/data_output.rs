use std::path::{PathBuf};
use std::fs;
use std::env::current_dir;
use crate::recommendation_generation::*;
use std::io::{self, Write};

fn create_directory_if_nonexistent(directory: &PathBuf) -> io::Result<()> {
	if !directory.exists() {
		fs::create_dir_all(&directory)?;
	}
	Ok(())
}

fn compute_data_directory() -> io::Result<PathBuf> {
	let mut data_directory = current_dir()?;
	data_directory.push("data");
	Ok(data_directory)
}

pub fn create_data_directory() -> io::Result<()> {
	let path = compute_data_directory()?;
	create_directory_if_nonexistent(&path)?;
	Ok(())
}



pub fn output_recommendations(
	recommendations: &[Information],
	file_name: &str,
) -> std::io::Result<()> {
	let mut file_path = compute_data_directory()?;
	file_path.push(file_name);

	let mut file = fs::File::create(file_path)?;
	let mut buffered_writer = io::BufWriter::new(file);


	for recommendation in recommendations {
		let mut number_of_words_saved = 0;

		let concrete_info = match recommendation {
			Information::Concrete(info) => {
				number_of_words_saved = info.get_number_of_words_saved();
				info
			},
			Information::Abstract(info) => {
				number_of_words_saved = info.get_number_of_words_saved();
				info.get_potential_command_information()
			}
		};
		writeln!(buffered_writer, "#Number of times used: {}", concrete_info.get_number_of_times_used())?;
		writeln!(buffered_writer, "#Number of words saved: {}", number_of_words_saved)?;
		if let Information::Abstract(info) = recommendation {
			writeln!(buffered_writer, "Number of instantiations of abstract command: {}", info.get_number_of_instantiations())?;
		}
		let actions = concrete_info.get_actions();
		actions.iter().for_each(|action| {
			let action_string = action.compute_talon_script();
			writeln!(buffered_writer, "{}", action_string).unwrap();
		});
		writeln!(buffered_writer, "")?;
		writeln!(buffered_writer, "")?;
	}

	Ok(())
}
