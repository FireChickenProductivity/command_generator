/*
class BasicAction:
	def __init__(self, name, arguments):
		self.name = name
		self.arguments = arguments
	
	def compute_talon_script(self):
		code = self.name + '(' + ', '.join(self.compute_arguments_converted_to_talon_script_string()) + ')'
		return code
	
	def compute_arguments_converted_to_talon_script_string(self):
		result = []
		for argument in self.arguments:
			if type(argument) == str:
				converted_argument = self.compute_string_argument(argument)
			elif type(argument) == bool:
				converted_argument = str(compute_talon_script_boolean_value(argument))
			else:
				converted_argument = str(argument)
			result.append(converted_argument)
		return result
	
	def compute_string_argument(self, argument: str):
		string_argument = "'" + argument.replace("'", "\\'") + "'"
		return string_argument
	
	def get_name(self):
		return self.name
	
	def get_arguments(self):
		return self.arguments
	
	def to_json(self) -> str:
		return json.dumps({'name': self.name, 'arguments': self.arguments}, cls = BasicActionEncoder)
	
	@staticmethod
	def from_json(text: str):
		representation = json.loads(text)
		return BasicAction(representation['name'], representation['arguments'])
	
	def __eq__(self, other) -> bool:
		return other is not None and self.name == other.name and self.arguments == other.arguments
	
	def __repr__(self):
		return self.__str__()
	
	def __str__(self):
		return self.to_json() */

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
		format!("<{}>", self.name)
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
			Argument::StringArgument(arg) => arg.to_string(),
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

	//I will have to think about how to handle talon captures
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

