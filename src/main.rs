mod input_parsing;

fn main() {
	let _file = input_parsing::get_file_from_user();
	println!("read file");
	let chain_size = input_parsing::get_max_chain_size_from_user();
	println!("read number {chain_size}");
	let number_of_recommendations = input_parsing::get_number_of_recommendations_from_user();
	println!("read number {number_of_recommendations}");
} 