mod input_parsing;
mod action_records;

use action_records::{BasicAction, Argument, Command};


fn main() {
	let action = action_records::BasicAction::new ("test", vec![Argument::StringArgument("test_string".to_string()), Argument::IntArgument(42), Argument::BoolArgument(true), Argument::FloatArgument(3.14), Argument::CaptureArgument(action_records::TalonCapture::new("capture_name", 1))],
	);
	let insert_action = action_records::BasicAction::new("insert", vec![Argument::StringArgument(String::from("text"))]);
	let key_action = action_records::BasicAction::new("key", vec![Argument::StringArgument(String::from("text"))]);
	let command = Command::new("test_command", vec![insert_action, key_action], None);
	println!("Action: {}", action.compute_talon_script());
	println!("json: {}", action.to_json());
	println!("command: {}", command.to_string());
	let parameters = input_parsing::get_input_parameters_from_user();
	println!("read number {}", parameters.max_chain_size);
	println!("read number {}", parameters.number_of_recommendations);
} 