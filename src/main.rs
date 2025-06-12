mod input_parsing;
mod action_records;
mod recommendation_generation;
mod action_utilities;
mod text_separation;

use action_records::{BasicAction, Argument, Command, read_file_record, Entry};
use recommendation_generation::{compute_recommendations_from_record, PotentialCommandInformation};
use std::time::Instant;

fn print_record(record: Result<Vec<Entry>, String>) {
	match record {
		Ok(record) => {
			for entry in record {
				match entry {
					Entry::RecordingStart => println!("Recording started."),
					Entry::Command(command) => {
						println!("Command: {}", command.to_string());
					}
				}
			}
		},
		Err(e) => println!("Error reading record file:\n	{}", e),
	}
}



fn main() {
	let parameters = input_parsing::get_input_parameters_from_user();
	let start_time = Instant::now();
	let record = read_file_record(parameters.record_file);
	match record {
		Ok(record) => {
			let elapsed_time = start_time.elapsed();
			compute_recommendations_from_record(&record, parameters.max_chain_size);
			println!("Time taken to compute recommendations: {:.3?}", elapsed_time);
		}
		Err(e) => println!("Error reading record file:\n	{}", e),
	}
} 