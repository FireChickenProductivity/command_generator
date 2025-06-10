use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

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

#[derive(Clone)]
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

#[derive(Clone)]
pub enum Entry {
	RecordingStart,
	Command(Command),
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

fn add_current_item(
	arguments: &mut Vec<Argument>,
	name: &mut String,
	key: &mut String,
	value_text: &mut String,
	is_current_value_string: &mut bool,
	is_inside_list: bool,
	capture_name: &mut String,
	capture_instance: &mut i32,
	number_of_unclosed_braces: &mut i32,
) -> Result<(), String> {
	if value_text.is_empty() {
		let message = format!("called add_current_item with key: |{}|, value_text: |{}|, is_current_value_string: {}\n", key, value_text, is_current_value_string);
		return Err(String::from(message));
	}

	if key == "name" {
		if *number_of_unclosed_braces == 2 {
			*capture_name = value_text.clone();
		} else {
			if !name.is_empty() {
				return Err(String::from("JSON string has multiple name fields"));
			}
			*name = value_text.clone();
		}
		key.clear();
	} else if key == "instance" {
		if *capture_instance != -1 {
			return Err(String::from("JSON string has multiple instance fields"));
		}
		if let Ok(instance) = value_text.parse::<i32>() {
			*capture_instance = instance;
		} else {
			return Err(format!("Invalid instance value: {}", value_text));
		}
		if *capture_instance < 0 {
			return Err(String::from("Instance value cannot be negative"));
		}
		key.clear();
		*capture_instance = -1;
	} else if is_inside_list {
		let argument = parse_basic_action_json_argument_element(value_text, *is_current_value_string)?;
		arguments.push(argument);
	} else {
		return Err(format!("JSON string has a key '{}' without a list", key));
	}

	value_text.clear();
	*is_current_value_string = false;
	Ok(())
}

fn load_basic_action_from_json(json: &str) -> Result<BasicAction, String> {
	let text = json.trim();
	let mut name = String::new();
	let mut arguments: Vec<Argument> = Vec::new();
	let mut key = String::new();
	let mut is_inside_string = false;
	let mut is_inside_list = false;
	let mut is_current_value_string = false;
	let mut current_text = String::new();
	let mut escape_next_character = false;
	let mut string_boundary = '"';
	let mut unclosed_opening_braces = 0;
	let mut capture_name = String::new();
	let mut capture_instance = -1;
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
			unclosed_opening_braces += 1;
		} else if char == '[' {
			if is_inside_list {
				return Err(String::from("JSON string has nested lists, which is not permitted"));
			} else if key != "arguments" {
				return Err(format!("JSON string has a list without a key 'arguments', found: {}", key));
			} 
			is_inside_list = true;
			if key.is_empty() {
				return Err(String::from("List encountered without a key"));
			}
			if unclosed_opening_braces != 1 {
				return Err(String::from("List encountered without containing map"));
			} 
		} else if char == '}' {
			if unclosed_opening_braces == 0 {
				return Err(String::from("JSON string has extraneous closing brace"));
			}
			unclosed_opening_braces -= 1;
			if is_inside_list {
				arguments.push(Argument::CaptureArgument(TalonCapture::new(&capture_name, capture_instance)));
				capture_name.clear();
				capture_instance = -1;
			}
			if !key.is_empty() || !current_text.is_empty() || is_current_value_string {
				add_current_item(&mut arguments, &mut name, &mut key, &mut current_text, &mut is_current_value_string, is_inside_list, &mut capture_name, &mut capture_instance, &mut unclosed_opening_braces)?;
			}
			
		} else if char == ']' {
			if !is_inside_list {
				return Err(String::from("JSON string has a closing bracket without an opening bracket"));
			}
			
			if !current_text.is_empty() {
				add_current_item(&mut arguments, &mut name, &mut key, &mut current_text, &mut is_current_value_string, is_inside_list, &mut capture_name, &mut capture_instance, &mut unclosed_opening_braces)?;
			}
			
			is_inside_list = false;
			key.clear();
		} else if char == ':' {
			if !key.is_empty() && !is_inside_list {
				return Err(format!("JSON string has a colon with a predefined key: {}", key));
			}
			if unclosed_opening_braces == 0 {
				return Err(String::from("JSON string has a colon without an opening brace"));
			}
			key = String::from(current_text.clone());
			current_text.clear();
			is_current_value_string = false;
		} else if char == ',' {
			add_current_item(&mut arguments, &mut name, &mut key, &mut current_text, &mut is_current_value_string, is_inside_list, &mut capture_name, &mut capture_instance, &mut unclosed_opening_braces)?;
		} else if char == '"' || char == '\'' {
			is_inside_string = true;
			string_boundary = char;
			is_current_value_string = true;
		} else if is_inside_list && (!current_text.is_empty() || char != ' ') {
			current_text.push(char);
		}
	}
	

	if is_inside_string {
		return Err(String::from("JSON string ends with an unclosed string"));
	} else if unclosed_opening_braces > 0 {
		return Err(String::from("JSON string ends with unclosed braces"));
	} else if is_inside_list {
		return Err(String::from("JSON string ends with an unclosed list"));
	} else if !key.is_empty() || !current_text.is_empty() || is_current_value_string {
		return Err(String::from("JSON string ends with an incomplete key-value pair"));
	}
	Ok(BasicAction::new(&name, arguments))
}

fn compute_command_name_without_prefix(name: &str) -> Result<String, String> {
	if name.starts_with(COMMAND_NAME_PREFIX) {
		let name_without_prefix = &name[COMMAND_NAME_PREFIX.len()..];
		if name_without_prefix.is_empty() {
			return Err(format!("Command name text is empty after removing prefix '{}'", COMMAND_NAME_PREFIX));
		}
		Ok(name_without_prefix.to_string())
	} else {
		let message = format!("Command name text does not start with prefix '{}': {}", COMMAND_NAME_PREFIX, name);
		Err(message)
	}
}

fn compute_seconds_since_last_action(time_record: &str) -> Result<u32, String> {
	if time_record.starts_with(TIME_DIFFERENCE_PREFIX) {
		let time_text = &time_record[TIME_DIFFERENCE_PREFIX.len()..];
		if let Ok(seconds) = time_text.parse::<u32>() {
			Ok(seconds)
		} else {
			Err(format!("Invalid time difference format: {}", time_text))
		}
	} else {
		Err(format!("Time record does not start with '{}': {}", TIME_DIFFERENCE_PREFIX, time_record))
	}
}

fn is_line_time_difference(line: &str) -> bool {
	line.starts_with(TIME_DIFFERENCE_PREFIX)
}

fn is_line_recording_start(line: &str) -> bool {
	line == RECORDING_START_MESSAGE
}

fn is_line_command_start(line: &str) -> bool {
	line.starts_with(COMMAND_NAME_PREFIX)
}

fn is_line_command_ending(line: &str) -> bool {
	is_line_command_start(line) || is_line_recording_start(line)
}

fn is_line_action(line: &str) -> bool {
	line.starts_with("{")
}



struct RecordParser <'a> {
	record: &'a mut Vec<Entry>,
	current_command_name: String,
	current_command_actions: Vec<BasicAction>,
	seconds_since_last_action: Option<u32>,
	seconds_since_last_action_for_next_command: Option<u32>,
	time_information_found_after_command: bool,
	line_number: usize,
}

impl <'a> RecordParser <'a> {
	pub fn new(record: &'a mut Vec<Entry>) -> Self {
		RecordParser {
			record: record,
			current_command_name: String::new(),
			current_command_actions: Vec::new(),
			seconds_since_last_action: None,
			seconds_since_last_action_for_next_command: None,
			time_information_found_after_command: false,
			line_number: 0,
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
			return Err(format!("Command #{} has no name", self.record.len() + 1));
		} else if self.current_command_actions.is_empty() {
			return Err(format!("Command ({}) #{} has no actions", self.current_command_name, self.record.len() + 1));
		}

		let seconds_since_last_action = self.compute_seconds_since_last_command();
		let command = Command::new(
			&self.current_command_name,
			self.current_command_actions.clone(),
			seconds_since_last_action,
		);
		self.record.push(Entry::Command(command));
		Ok(())
	}

	fn is_command_found(&self) -> bool {
		!self.current_command_name.is_empty() && !self.current_command_actions.is_empty()
	}

	fn add_action_based_on_line(&mut self, line: &str) -> Result<(), String> {
		let action = load_basic_action_from_json(line)?;
		self.current_command_actions.push(action);
		Ok(())
	}

	fn add_current_command_if_available(&mut self) -> Result<(), String> {
		if self.is_command_found() {
			self.add_current_command()?;
		}
		Ok(())
	}
	
	fn process_command_start(&mut self, line: &str) -> Result<(), String> {
		self.add_current_command_if_available()?;
		self.current_command_name = compute_command_name_without_prefix(line)?;
		Ok(())
	}

	fn process_time_difference(&mut self, line: &str) -> Result<(), String> {
		self.seconds_since_last_action = self.seconds_since_last_action_for_next_command;
		self.seconds_since_last_action_for_next_command = Some(compute_seconds_since_last_action(line)?);
		self.time_information_found_after_command = true;
		Ok(())
	}

	fn process_recording_start(&mut self) -> Result<(), String> {
		self.add_current_command_if_available()?;
		self.record.push(Entry::RecordingStart);
		self.current_command_name.clear();
		Ok(())
	}

	fn reset_command_information_except_name(&mut self) {
		self.current_command_actions.clear();
		self.seconds_since_last_action = None;
		self.seconds_since_last_action_for_next_command = None;
		self.time_information_found_after_command = false;
	}

	fn parse_line(&mut self, line: &str) -> Result<(), String> {
		if is_line_action(line) {
			self.add_action_based_on_line(line)?;
		} else if is_line_command_start(line) {
			self.process_command_start(line)?;
		} else if is_line_time_difference(line) {
			self.process_time_difference(line)?;
		} else if is_line_recording_start(line) {
			self.process_recording_start()?;
		} 
		if is_line_command_ending(line) {
			self.reset_command_information_except_name();
		} 
		Ok(())
	}

	fn parse_file_lines(&mut self, file: io::BufReader<File>) -> Result<(), String> {
		for line in file.lines().map_while(Result::ok) {
			self.line_number += 1;
			if let Err(message) = self.parse_line(line.trim()) {
				return Err(format!("Error parsing line ({})\n	{}", line, message));
			}
		}
		Ok(())
	}

	pub fn parse_file(&mut self, file: io::BufReader<File>) -> Result<(), String> {
		if let Err(message) = self.parse_file_lines(file) {
			return Err(format!("Error parsing file at line {}: {}", self.line_number, message));
		}
		if self.is_command_found() {
			self.add_current_command()?;
		}
		Ok(())
	}
}

pub fn read_file_record(file: File) -> Result<Vec<Entry>, String> {
	let reader = io::BufReader::new(file);
	let mut record: Vec<Entry> = Vec::new();
	let mut parser = RecordParser::new(&mut record);
	parser.parse_file(reader)?;
	Ok(record)
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

	fn assert_actions_match(action: &BasicAction, name: &str, arguments: &Vec<Argument>) {
		assert_eq!(action.get_name(), name, "Action name does not match");
		assert_eq!(action.get_arguments().len(), arguments.len(), "Number of arguments do not match");
		for (i, expected_argument) in arguments.iter().enumerate() {
			let actual_argument = &action.get_arguments()[i];
			assert_eq!(expected_argument, actual_argument, "Argument at index {} does not match", i);
		}
		
	}
	
	fn assert_action_matches_expected_from_string(
		name: &str,
		arguments: &Vec<Argument>,
		text: &str,
	) {
		
		let actual_result = load_basic_action_from_json(text);
		match actual_result {
			Ok(actual) => {
				assert_actions_match(&actual, name, arguments);
			}
			Err(message) => {
				panic!("Error parsing JSON:\n    {}", message);
			}
		}
	}
	
	#[test]
	fn test_insert_action() {
		let name = String::from("insert");
		let arguments = vec![Argument::StringArgument(String::from("text"))];
		let json = r#"{"name": "insert", "arguments": ["text"]}"#;
		assert_action_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_insert_capture_action() {
		let name = String::from("insert");
		let arguments = vec![Argument::CaptureArgument(TalonCapture::new("capture_name", 1))];
		let json = r#"{"name": "insert", "arguments": [{"name": "capture_name", "instance": 1}]}"#;
		assert_action_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_mouse_move_action() {
		let name = String::from("mouse_move");
		let arguments = vec![
			Argument::IntArgument(100),
			Argument::IntArgument(200),
		];
		let json = r#"{"name": "mouse_move", "arguments": [100, 200]}"#;
		assert_action_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_mouse_click_action() {
		let name = String::from("mouse_click");
		let arguments = vec![
			Argument::IntArgument(1),
		];
		let json = r#"{"name": "mouse_click", "arguments": [1]}"#;
		assert_action_matches_expected_from_string(&name, &arguments, json);
	}

	#[test]
	fn test_mouse_scroll_action() {
		let name = String::from("mouse_scroll");
		let arguments = vec![
			Argument::IntArgument(0),
			Argument::IntArgument(1),
			Argument::BoolArgument(true),
		];
		let json = r#"{"name": "mouse_scroll", "arguments": [0, 1, true]}"#;
		assert_action_matches_expected_from_string(&name, &arguments, json);
	}

}