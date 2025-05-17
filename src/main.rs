mod input_parsing;
mod action_records;

fn main() {
	let action = action_records::BasicAction {
		name: String::from("test"),
		arguments: vec![String::from("arg1"), String::from("arg2")],
	};
	println!("Action: {}", action.compute_talon_script());
	println!("json: {}", action.to_json());

	let parameters = input_parsing::get_input_parameters_from_user();
	println!("read number {}", parameters.max_chain_size);
	println!("read number {}", parameters.number_of_recommendations);
} 