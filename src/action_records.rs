use std::collections::HashMap;
use std::fs::File;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Debug)]
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
#[derive(Debug)]
pub enum Argument {
	StringArgument(String),
	IntArgument(i32),
	BoolArgument(bool),
	FloatArgument(f64),
	CaptureArgument(TalonCapture),
}

impl PartialEq for Argument {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Argument::StringArgument(a), Argument::StringArgument(b)) => a == b,
			(Argument::IntArgument(a), Argument::IntArgument(b)) => a == b,
			(Argument::BoolArgument(a), Argument::BoolArgument(b)) => a == b,
			(Argument::FloatArgument(a), Argument::FloatArgument(b)) => a == b,
			(Argument::CaptureArgument(a), Argument::CaptureArgument(b)) => a == b,
			_ => false,
		}
	}
	
}

#[derive(Clone)]
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

fn parse_basic_action_json_argument_element(text: &str, is_string: bool) -> Result<Argument, String> {
	if is_string {
		return Ok(Argument::StringArgument(String::from(text)));
	}
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
		let message = format!("Invalid JSON element: {}, is string: {}", trimmed_text, is_string);
		Err(String::from(message))
	}
}

#[derive(Debug)]
enum JsonElement {
	Argument(Argument),
	String(String),
	Container(JsonContainer),
}

#[derive(Debug)]
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
	is_current_value_string: &mut bool,
) -> Result<(), String> {
	if stack.is_empty() {
		return Err(String::from("JSON string has no open container to add item to"));
	} else if value_text.is_empty() {
		let message = format!("called add_current_item with key: |{}|, value_text: |{}|, is_current_value_string: {}\n", key, value_text, is_current_value_string);
		return Err(String::from(message));
	}

	match stack.last_mut().unwrap() {
		JsonContainer::HashMap(map) => {
			if key.is_empty() {
				return Err(String::from("JSON string has empty key for item"));
			}
			let argument = JsonElement::Argument(parse_basic_action_json_argument_element(value_text, *is_current_value_string)?);
			map.insert(key.clone(), argument);
			key.clear();
		}
		JsonContainer::Arguments(arguments) => {
			let argument = parse_basic_action_json_argument_element(value_text, *is_current_value_string)?;
			arguments.push(argument);
		}
	}
	value_text.clear();
	*is_current_value_string = false;
	Ok(())
}

fn load_talon_capture_from_map(map: &HashMap<String, JsonElement>) -> Result<TalonCapture, String> {
	let name = match map.get("name") {
		Some(JsonElement::Argument(Argument::StringArgument(name))) => name,
		_ => return Err(format!("Capture JSON does not contain a name field {:?}", map)),
	};
	match map.get("instance") {
		Some(JsonElement::Argument(Argument::IntArgument(instance))) => {
			return Ok(TalonCapture::new(name, *instance));
		}
		_ => return Err(format!("Capture JSON does not contain an instance field {:?}", map)),
	};
}

fn load_basic_action_map_from_json(json: &str) -> Result<HashMap<String, JsonElement>, String> {
	let mut stack: Vec<JsonContainer> = Vec::new();
	let text = json.trim();
	let mut key = String::new();
	let mut is_inside_string = false;
	let mut is_inside_list = false;
	let mut argument_key = String::new();
	let mut is_current_value_string = false;
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
			is_inside_list = true;
			if key.is_empty() {
				return Err(String::from("List encountered without a key"));
			} else {
				argument_key = key.clone();
				key.clear();
			}
			if stack.len() < 1 {
				return Err(String::from("List encountered without containing map"));
			} else {
				stack.push(JsonContainer::Arguments(Vec::new()));
			}
		} else if char == '}' {
			if stack.is_empty() {
				return Err(String::from("JSON string has extraneous closing brace"));
			}
			if !key.is_empty() || !current_text.is_empty() || is_current_value_string {
				add_current_item(&mut stack, &mut key, &mut current_text, &mut is_current_value_string)?;
			}
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
			is_inside_list = false;
			if stack.len() < 2 {
				return Err(String::from("JSON string has extraneous closing bracket"));
			} else {
				if !current_text.is_empty() {
					add_current_item(&mut stack, &mut key, &mut current_text, &mut is_current_value_string)?;
				}
				let container = stack.pop().unwrap();
				if let JsonContainer::Arguments(arguments) = container {
					if let JsonContainer::HashMap(map) = stack.last_mut().unwrap() {
						if argument_key.is_empty() {
							return Err(String::from("JSON string has empty key for arguments"));
						} else {
							map.insert(argument_key.clone(), JsonElement::Container(JsonContainer::Arguments(arguments)));
							argument_key.clear();
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
				return Err(format!("JSON string has a colon with a predefined key: {}", key));
			}
			match stack.last_mut() {
				Some(JsonContainer::HashMap(_)) => {
					key = String::from(current_text.clone());
					current_text.clear();
					is_current_value_string = false;
				}
				_ => return Err(String::from("JSON string has a colon without a containing map")),
			}
		} else if char == ',' {
			add_current_item(&mut stack, &mut key, &mut current_text, &mut is_current_value_string)?;
		} else if char == '"' || char == '\'' {
			is_inside_string = true;
			string_boundary = char;
			is_current_value_string = true;
		} else if is_inside_list && (!current_text.is_empty() || char != ' ') {
			current_text.push(char);
		}
	}
	

	handle_stack_result(&mut stack)
}

fn load_basic_action_from_json(json: &str) -> Result<BasicAction, String> {
	let map = load_basic_action_map_from_json(json)?;
	let name = match map.get("name") {
		Some(JsonElement::String(name)) => name,
		_ => return Err(String::from("JSON does not contain a name field")),
	};
	let arguments = match map.get("arguments") {
		Some(JsonElement::Container(JsonContainer::Arguments(args))) => args,
		_ => return Err(String::from("JSON does not contain an arguments field")),
	};
	let action = BasicAction::new(name, arguments.clone());
	Ok(action)
}

struct RecordParser {
	file: File,
	commands: Vec<Command>,
	current_command_name: String,
	current_command_actions: Vec<BasicAction>,
	seconds_since_last_action: Option<u32>,
	seconds_since_last_action_for_next_command: Option<u32>,
	time_information_found_after_command: bool,
}

impl RecordParser {
	pub fn new(input_file: File) -> Self {
		RecordParser {
			file: input_file,
			commands: Vec::new(),
			current_command_name: String::new(),
			current_command_actions: Vec::new(),
			seconds_since_last_action: None,
			seconds_since_last_action_for_next_command: None,
			time_information_found_after_command: false,
		}
	}

	fn compute_seconds_since_last_command(&self) -> Option<u32> {
		if self.time_information_found_after_command {
			self.seconds_since_last_action_for_next_command
		} else {
			self.seconds_since_last_action
		}
	}
	
	fn add_current_command(&mut self) -> Result<(), String> {
		if self.current_command_name.is_empty() {
			return Err(format!("Command #{} has no name", self.commands.len() + 1));
		} else if self.current_command_actions.is_empty() {
			return Err(format!("Command ({}) #{} has no actions", self.current_command_name, self.commands.len() + 1));
		}

		let seconds_since_last_action = self.compute_seconds_since_last_command();
		let command = Command::new(
			&self.current_command_name,
			self.current_command_actions.clone(),
			seconds_since_last_action,
		);
		self.commands.push(command);
		Ok(())
	}

	fn is_command_found(&self) -> bool {
		!self.current_command_name.is_empty() && !self.current_command_actions.is_empty()
	}

	fn process_line(&mut self, line: String) -> Result<(), String> {
		// Placeholder
		Ok(())
	}

	fn parse_file_lines(&mut self) -> Result<(), String> {
		// Placeholder
		Ok(())
	}

	pub fn parse_file(&mut self) -> Result<(), String> {
		self.parse_file_lines();
		if self.is_command_found() {
			self.add_current_command()?;
		}
		Ok(())
	}
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

	fn assert_keys_match(
		expected: &HashMap<String, JsonElement>,
		actual: &HashMap<String, JsonElement>,
	) {
		for key in expected.keys() {
			assert!(actual.contains_key(key), "Key {} not found in actual map", key);
		}
		for key in actual.keys() {
			assert!(expected.contains_key(key), "Key {} not found in expected map", key);
		}
	}

	fn treat_json_element_as_string_with_panic(element: &JsonElement) -> &String {
		match element {
			JsonElement::String(s) => s,
			JsonElement::Argument(arg) => match arg {
				Argument::StringArgument(s) => s,
				_ => panic!("Expected a string argument, found something else"),
			},
			JsonElement::Container(_) => panic!("Expected a string, found a container"),
		}
	}

	fn treat_json_element_as_arguments_with_panic(
		element: &JsonElement,
	) -> &Vec<Argument> {
		match element {
			JsonElement::Container(JsonContainer::Arguments(args)) => args,
			_ => panic!("Expected an arguments container, found something else"),
		}
	}

	fn assert_map_content_match(
		expected: &HashMap<String, JsonElement>,
		actual: &HashMap<String, JsonElement>,
	) {
		let expected_name = treat_json_element_as_string_with_panic(
			expected.get("name").unwrap()
		);
		let actual_name = treat_json_element_as_string_with_panic(
			actual.get("name").unwrap()
		);
		assert_eq!(expected_name, actual_name, "Action names do not match");
		let expected_arguments = treat_json_element_as_arguments_with_panic(
			expected.get("arguments").unwrap()
		);
		let actual_arguments = treat_json_element_as_arguments_with_panic(
			actual.get("arguments").unwrap()
		);
		assert_eq!(expected_arguments.len(), actual_arguments.len(), "Number of arguments do not match");
		for (i, expected_argument) in expected_arguments.iter().enumerate() {
			let actual_argument = &actual_arguments[i];
			assert_eq!(expected_argument, actual_argument, "Argument at index {} does not match", i);
		}
	}

	fn assert_maps_match(
		expected: &HashMap<String, JsonElement>,
		actual: &HashMap<String, JsonElement>,
	) {
		assert_keys_match(expected, actual);
		assert_map_content_match(expected, actual);
	}
	
	fn assert_map_matches_expected_from_string(
		name: &str,
		arguments: &Vec<Argument>,
		text: &str,
	) {
		let mut expected = HashMap::new();
		expected.insert(
			String::from("name"),
			JsonElement::String(String::from(name)),
		);
		expected.insert(
			String::from("arguments"),
			JsonElement::Container(JsonContainer::Arguments(arguments.clone())),
		);
		let actual_result = load_basic_action_map_from_json(text);
		match actual_result {
			Ok(actual) => {
				assert_eq!(actual.len(), expected.len());
				assert_maps_match(&expected, &actual);
			}
			Err(message) => {
				panic!("Error parsing JSON: {}", message);
			}
		}
	}
	
	#[test]
	fn test_insert_map() {
		let name = String::from("insert");
		let arguments = vec![Argument::StringArgument(String::from("text"))];
		let json = r#"{"name": "insert", "arguments": ["text"]}"#;
		assert_map_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_insert_capture_map() {
		let name = String::from("insert");
		let arguments = vec![Argument::CaptureArgument(TalonCapture::new("capture_name", 1))];
		let json = r#"{"name": "insert", "arguments": [{"name": "capture_name", "instance": 1}]}"#;
		assert_map_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_mouse_move_map() {
		let name = String::from("mouse_move");
		let arguments = vec![
			Argument::IntArgument(100),
			Argument::IntArgument(200),
		];
		let json = r#"{"name": "mouse_move", "arguments": [100, 200]}"#;
		assert_map_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_mouse_click_map() {
		let name = String::from("mouse_click");
		let arguments = vec![
			Argument::IntArgument(1),
		];
		let json = r#"{"name": "mouse_click", "arguments": [1]}"#;
		assert_map_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_mouse_scroll_map() {
		let name = String::from("mouse_scroll");
		let arguments = vec![
			Argument::IntArgument(0),
			Argument::IntArgument(1),
			Argument::BoolArgument(true),
		];
		let json = r#"{"name": "mouse_scroll", "arguments": [0, 1, true]}"#;
		assert_map_matches_expected_from_string(&name, &arguments, json);
	}

}