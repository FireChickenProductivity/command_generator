mod input_parsing;
mod action_records;

use action_records::{BasicAction, Argument};


fn main() {
	let action = action_records::BasicAction::new ("test", vec![Argument::StringArgument("test_string".to_string()), Argument::IntArgument(42), Argument::BoolArgument(true), Argument::FloatArgument(3.14), Argument::CaptureArgument(action_records::TalonCapture::new("capture_name", 1))],
	);
	println!("Action: {}", action.compute_talon_script());
	println!("json: {}", action.to_json());
	let parameters = input_parsing::get_input_parameters_from_user();
	println!("read number {}", parameters.max_chain_size);
	println!("read number {}", parameters.number_of_recommendations);
} 