mod input_parsing;

fn main() {
	let parameters = input_parsing::get_input_parameters_from_user();
	println!("read number {}", parameters.max_chain_size);
	println!("read number {}", parameters.number_of_recommendations);
} 