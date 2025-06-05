mod input_parsing;
mod action_records;

use action_records::{BasicAction, Argument, Command, read_file_record, Entry};


fn main() {
	let parameters = input_parsing::get_input_parameters_from_user();
	let record = read_file_record(parameters.record_file);
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
		Err(e) => println!("Error reading record file: {}", e),
	}
} 