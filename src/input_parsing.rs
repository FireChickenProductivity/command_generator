use std::io;
use std::fs::File;
use std::env;

pub fn get_file_from_user() -> File {
	return loop {
		println!("Input the filepath to the command record: ");
		let mut input = String::new();
		let _result = io::stdin().read_line(&mut input);
		match _result {
			Ok(_result) => {
				let _file_result = File::open(input.trim());
				match _file_result {
					Ok(valid_file) => break valid_file,
					Err(_) => println!("Please input a valid path."),
				}
			}
			Err(_) => println!("Error reading input!"),
		}
	};
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
				} else if let Ok(value) = trimmed_input.parse() {
					return value;
				} else {
					println!("Please enter a non-negative integer.");
				}
			}
			Err(_) => println!("Error reading input!"),
		}
	}
}

pub fn get_max_chain_size_from_user() -> u32 {
	get_nonnegative_integer_from_user("Input the maximum number of consecutive commands to consider as a single potential command.\nMaking this bigger can allow finding longer patterns but it takes longer. Press enter with no input to take default of 20: ", 20)
}

pub fn get_number_of_recommendations_from_user() -> u32 {
	get_nonnegative_integer_from_user("Input the maximum number of command recommendations to output. Press enter with no input to take default of 0: ", 0)
}

pub struct InputParameters {
	pub record_file: File,
	pub max_chain_size: u32,
	pub number_of_recommendations: u32,
}

fn get_file(arguments: &Vec<String>) -> File {
	if arguments.len() < 2 {
		get_file_from_user()
	} else if let Ok(file) = File::open(&arguments[1]) {
		file
	} else {
		println!("Could not open the record file.");
		get_file_from_user()
	}
}

fn get_max_chain_size(arguments: &Vec<String>) -> u32 {
	if arguments.len() < 3 {
		get_max_chain_size_from_user()
	} else if let Ok(size) = arguments[2].parse() {
			size
	} else {
		println!("Could not parse the maximum chain size.");
		get_max_chain_size_from_user()
	}
}

fn get_number_of_recommendations(arguments: &Vec<String>) -> u32 {
	if arguments.len() < 4 {
		get_number_of_recommendations_from_user()
	} else if let Ok(size) = arguments[3].parse() {
			size
	} else {
		println!("Could not parse the number of recommendations.");
		get_number_of_recommendations_from_user()
	}
}

pub fn get_input_parameters_from_user() -> InputParameters {
	let arguments: Vec<String> = env::args().collect();
	let record_file = get_file(&arguments);
	let max_chain_size = get_max_chain_size(&arguments);
	let number_of_recommendations = get_number_of_recommendations(&arguments);

	InputParameters {
		record_file,
		max_chain_size,
		number_of_recommendations,
	}
}