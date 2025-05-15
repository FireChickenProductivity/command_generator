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
						println!("Received invalid filepath!");
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

