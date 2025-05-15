mod input_parsing;

fn main() {
	let _file = input_parsing::get_file_from_user();
	println!("read file");
	let number = input_parsing::get_nonnegative_integer_from_user("Input the command chain size: ", 20);
	println!("read number {number}");
} 