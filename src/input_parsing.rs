use std::io;
use std::fs::File;

pub fn get_file_from_user() -> File {
	let file = loop {
		println!("Input the filepath to the command record: ");
		let mut input = String::new();
		let _result = io::stdin().read_line(&mut input);
		match _result {
			Ok(_result) => {
				let _file_result = File::open(input.trim());
				match _file_result {
					Ok(valid_file) => {
						break valid_file;
					}
					Err(_) => {
						println!("Please input a valid path.");
					}
				}
			}
			Err(_) => {
				println!("Error reading input!");
			}
		}
	};
	file
}

fn get_nonnegative_integer_from_user(prompt: &str, default: u32) -> u32 {
	loop {
		println!("{}", prompt);
		let mut input = String::new();
		let _result = io::stdin().read_line(&mut input);
		match _result {
			Ok(_result) => {
				let trimmed_input = input.trim();
				if trimmed_input.is_empty() {
					return default;
				} else if let Ok(value) = trimmed_input.parse::<u32>() {
					return value;
				} else {
					println!("Please enter a non-negative integer.");
				}
			}
			Err(_) => {
				println!("Error reading input!");
			}
		}
	}
}

pub fn get_max_chain_size_from_user() -> u32 {
	get_nonnegative_integer_from_user("Input the maximum number of consecutive commands to consider as a single potential command.\nMaking this bigger can allow finding longer patterns but it takes longer. Press enter with no input to take default of 20: ", 20)
}

pub fn get_number_of_recommendations_from_user() -> u32 {
	get_nonnegative_integer_from_user("Input the maximum number of command recommendations to output. Press enter with no input to take default of 0: ", 0)
}