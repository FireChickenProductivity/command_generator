use std::collections::HashMap;

#[derive(Clone)]
pub struct TalonCapture {
	name: String,
	instance: i32,
}

impl TalonCapture {
	pub fn new(name: &str, instance: i32) -> Self {
		TalonCapture {
			name: String::from(name),
			instance,
		}
	}

	pub fn compute_string_representation(&self) -> String {
		format!("{}_{}", self.name, self.instance)
	}

	pub fn compute_command_component(&self) -> String {
		format!("<{}_{}>", self.name, self.instance)
	}

	pub fn to_json(&self) -> String {
		format!("{{\"name\": \"{}\", \"instance\": {}}}", self.name, self.instance)
	}

	pub fn clone(&self) -> Self {
		TalonCapture {
			name: self.name.clone(),
			instance: self.instance,
		}
	}
}

#[derive(Clone)]
pub enum Argument {
	StringArgument(String),
	IntArgument(i32),
	BoolArgument(bool),
	FloatArgument(f64),
	CaptureArgument(TalonCapture),
}


pub struct BasicAction {
	name: String,
	arguments: Vec<Argument>,
}

impl BasicAction {
	pub fn new(name: &str, arguments: Vec<Argument>) -> Self {
		BasicAction {
			name: String::from(name),
			arguments,
		}
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}

	pub fn get_arguments(&self) -> &Vec<Argument> {
		&self.arguments
	}

	pub fn compute_talon_script(&self) -> String {
		let arguments_text = self.compute_arguments_converted_to_talon_script_string()
			.join(", ");
		let code = format!("{}({})", self.name, arguments_text);
		code
	}

	pub fn compute_arguments_converted_to_talon_script_string(&self) -> Vec<String> {
		self.arguments.iter()
			.map(|argument| self.compute_string_argument(argument))
			.collect()
	}

	pub fn compute_string_argument(&self, argument: &Argument) -> String {
		match argument {
			Argument::StringArgument(arg) => {
				let mut result = arg.to_string();
				if result.contains('\"') {
					result = result.replace("\"", "\\\"");
				}
				format!("\"{}\"", result)
			},
			Argument::IntArgument(arg) => arg.to_string(),
			Argument::BoolArgument(arg) => arg.to_string(),
			Argument::FloatArgument(arg) => arg.to_string(),
			Argument::CaptureArgument(arg) => arg.compute_command_component(),
		}
	}

	pub fn compute_argument_json(&self, argument: &Argument) -> String {
		match argument {
			Argument::StringArgument(arg) => arg.replace("\"", "\\\""),
			Argument::CaptureArgument(arg) => arg.to_json(),
			other => self.compute_string_argument(other),
		}
	}

	pub fn to_json(&self) -> String {
		let mut result = format!("{{\"name\": \"{}\", \"arguments\": [", self.name);
		let mut pushed_first = false;
		for argument in &self.arguments {
			if pushed_first {
				result.push_str(", ");
			} else {
				pushed_first = true;
			}
			result.push_str(self.compute_argument_json(argument).as_str());
		}
		result.push_str("]}");
		result
	}
	
	pub fn clone(&self) -> Self {
		BasicAction {
			name: self.name.clone(),
			arguments: self.arguments.clone(),
		}
	}
}

pub struct Command {
	name: String,
	actions: Vec<BasicAction>,
	seconds_since_last_action: Option<u32>,
}

impl Command {
	pub fn new(name: &str, actions: Vec<BasicAction>, seconds_since_last_action: Option<u32>) -> Self {
		Command {
			name: String::from(name),
			actions,
			seconds_since_last_action: seconds_since_last_action,
		}
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}

	pub fn get_actions(&self) -> &Vec<BasicAction> {
		&self.actions
	}

	pub fn get_seconds_since_last_action(&self) -> Option<u32> {
		self.seconds_since_last_action
	}

	pub fn to_string(&self) -> String {
		let actions_text: Vec<String> = self.actions.iter()
			.map(|action| action.to_json())
			.collect();
		let actions_joined = actions_text.join("");
		let seconds_since_last_text = match self.seconds_since_last_action {
			Some(seconds) => seconds.to_string(),
			None => String::new(),
		};
		format!("Command({}, {}, {}", self.name, seconds_since_last_text, actions_joined)
	}

	pub fn append(&mut self, command: &Command) {
		command.actions.iter().for_each(|action| {
			self.actions.push(action.clone());
		});
		self.name.push_str(command.get_name());
	}
}

struct CommandChain {
	command: Command,
	chain_number: u32,
	chain_size: u32,
}

impl CommandChain {
	pub fn new(command: Command, chain_number: u32, chain_size: u32) -> Self {
		CommandChain {
			command,
			chain_number,
			chain_size,
		}
	}

	pub fn get_command(&self) -> &Command {
		&self.command
	}

	pub fn get_chain_number(&self) -> u32 {
		self.chain_number
	}

	pub fn get_chain_ending_index(&self) -> u32 {
		self.chain_number + self.chain_size - 1
	}

	pub fn get_next_chain_index(&self) -> u32 {
		self.chain_number + self.chain_size
	}

	pub fn get_size(&self) -> u32 {
		self.chain_size
	}

	pub fn append_command(&mut self, command: Command) {
		self.command.append(&command);
		self.chain_size += 1;
	}
}

pub enum Entry {
	RecordingStart,
	Command,
}

const COMMAND_NAME_PREFIX: &str = "Command: ";
const RECORDING_START_MESSAGE: &str = "START";
const TIME_DIFFERENCE_PREFIX: &str = "T";

fn parse_basic_action_json_argument_element(text: &str) -> Result<Argument, String> {
	let trimmed_text = text.trim();
	if let Ok(i32) = trimmed_text.parse::<i32>() {
		Ok(Argument::IntArgument(i32))
	} else if let Ok(f64) = trimmed_text.parse::<f64>() {
		Ok(Argument::FloatArgument(f64))
	} else if trimmed_text == "true" {
		Ok(Argument::BoolArgument(true))
	} else if trimmed_text == "false" {
		Ok(Argument::BoolArgument(false))
	} else {
		Err(String::from("Invalid JSON element"))
	}
}

enum JsonElement {
	Argument(Argument),
	String(String),
	Container(JsonContainer),
}

enum JsonContainer {
	Arguments(Vec<Argument>),
	HashMap(HashMap<String, JsonElement>),
}

fn handle_stack_result(stack: &mut Vec<JsonContainer>) -> Result<HashMap<String, JsonElement>, String> {
	if stack.is_empty() {
		Err(String::from("JSON string is empty or not properly formatted."))
	} else if stack.len() > 1 {
		Err(String::from("JSON string has unclosed container"))
	} else {
		let result = stack.pop().unwrap();
		match result {
			JsonContainer::HashMap(map) => {
				Ok(map)
			}
			_ => Err(String::from("JSON string does not represent a valid action map.")),
		}
	}
}

fn add_current_item(
	stack: &mut Vec<JsonContainer>,
	key: &mut String,
	value_text: &mut String,
) -> Result<(), String> {
	if stack.is_empty() {
		return Err(String::from("JSON string has no open container to add item to"));
	} else if value_text.is_empty() {
		return Err(String::from("JSON string has no value for item"));
	}

	match stack.last_mut().unwrap() {
		JsonContainer::HashMap(map) => {
			if key.is_empty() {
				return Err(String::from("JSON string has empty key for item"));
			}
			let argument = JsonElement::Argument(parse_basic_action_json_argument_element(value_text)?);
			map.insert(key.clone(), argument);
			key.clear();
		}
		JsonContainer::Arguments(arguments) => {
			let argument = parse_basic_action_json_argument_element(value_text)?;
			arguments.push(argument);
		}
	}
	value_text.clear();
	Ok(())
}

fn load_talon_capture_from_map(map: &HashMap<String, JsonElement>) -> Result<TalonCapture, String> {
	let name = match map.get("name") {
		Some(JsonElement::String(name)) => name,
		_ => return Err(String::from("Capture JSON does not contain a name field")),
	};
	match map.get("instance") {
		Some(JsonElement::Argument(Argument::IntArgument(instance))) => {
			return Ok(TalonCapture::new(name, *instance));
		}
		_ => return Err(String::from("Capture JSON does not contain an instance field")),
	};
}

fn load_basic_action_map_from_json(json: &str) -> Result<HashMap<String, JsonElement>, String> {
	let mut stack: Vec<JsonContainer> = Vec::new();
	let text = json.trim();
	let mut key = String::new();
	let mut value_text = String::new();
	let mut is_inside_string = false;
	let mut current_text = String::new();
	let mut escape_next_character = false;
	let mut string_boundary = '"';
	for char in text.chars() {
		if is_inside_string {
			if char == '\\' {
				if escape_next_character {
					current_text.push(char);
					escape_next_character = false;
				} else {
					escape_next_character = true;
				}
			} else if char == string_boundary {
				if escape_next_character {
					current_text.push(char);
					escape_next_character = false;
				} else {
					is_inside_string = false;
				}
			} else {
				current_text.push(char);
				
			}
		} else if char == '{' {
			stack.push(JsonContainer::HashMap(HashMap::new()));
		} else if char == '[' {
			if stack.len() < 1 {
				return Err(String::from("List encountered without containing map"));
			} else {
				stack.push(JsonContainer::Arguments(Vec::new()));
			}
		} else if char == '}' {
			if stack.is_empty() {
				return Err(String::from("JSON string has extraneous closing brace"));
			}
			add_current_item(&mut stack, &mut key, &mut value_text)?;
			if stack.len() == 1 {
				if let JsonContainer::Arguments(_) = stack.last().unwrap() {
					return Err(String::from("JSON string has a list at the top level, which is not permitted"));
				}
			} else {
				let container = stack.pop().unwrap();
				if let JsonContainer::HashMap(map) = container {
					if stack.len() > 0 {
						if let JsonContainer::Arguments(arguments) = stack.last_mut().unwrap() {
							let capture = load_talon_capture_from_map(&map)?;
							arguments.push(Argument::CaptureArgument(capture))
						}
					} else {
						return Err(String::from("Found a map not contained by a list, which is not permitted"));
					}
				} else {
					return Err(String::from("JSON string has mismatched braces"));
				}
			}
		} else if char == ']' {
			if stack.len() < 2 {
				return Err(String::from("JSON string has extraneous closing bracket"));
			} else {
				add_current_item(&mut stack, &mut key, &mut value_text)?;
				let container = stack.pop().unwrap();
				if let JsonContainer::Arguments(arguments) = container {
					if let JsonContainer::HashMap(map) = stack.last_mut().unwrap() {
						if !key.is_empty() {
							map.insert(key.clone(), JsonElement::Container(JsonContainer::Arguments(arguments)));
							key.clear();
						} else {
							return Err(String::from("JSON string has empty key for arguments"));
						}
					} else {
						return Err(String::from("JSON string has mismatched brackets"));
					}
				} else {
					return Err(String::from("JSON string has mismatched brackets"));
				}
			}
		} else if char == ':' {
			if !key.is_empty() {
				return Err(String::from("JSON string has a colon without a key"))
			}
			match stack.last_mut() {
				Some(JsonContainer::HashMap(_)) => {
					key = String::from(current_text.clone());
					current_text.clear();
				}
				_ => return Err(String::from("JSON string has a colon without a containing map")),
			}
		} else if char == ',' {
			add_current_item(&mut stack, &mut key, &mut value_text)?;
		} else if char == '"' || char == '\'' {
			is_inside_string = true;
			string_boundary = char;
		}
	}
	

	handle_stack_result(&mut stack)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_action_string() {
		let action = BasicAction::new(
			"text_action",
			vec![
				Argument::StringArgument(String::from("text")),
				Argument::IntArgument(42),
				Argument::BoolArgument(true),
				Argument::FloatArgument(3.14),
				Argument::CaptureArgument(TalonCapture::new("capture_name", 1)),
			]
		);
		let talon_script = String::from("text_action(\"text\", 42, true, 3.14, <capture_name_1>)");
		assert_eq!(action.compute_talon_script(), talon_script);
	}
}