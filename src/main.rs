mod input_parsing;
mod action_records;

use action_records::{BasicAction, Argument, Command, read_file_record, Entry};
use std::time::Instant;

fn main() {
	let parameters = input_parsing::get_input_parameters_from_user();
	let start_time = Instant::now();
	let record = read_file_record(parameters.record_file);
	let elapsed_time = start_time.elapsed();
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
	println!("Elapsed time: {:.2?}", elapsed_time);
} 