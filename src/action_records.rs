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

pub struct BasicAction {
	name: String,
	arguments: Vec<String>,
}

impl BasicAction {
	pub fn new(name: &str, arguments: Vec<String>) -> Self {
		BasicAction {
			name: String::from("name"),
			arguments,
		}
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}

	pub fn get_arguments(&self) -> &Vec<String> {
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

	pub fn compute_string_argument(&self, argument: &str) -> String {
		let string_argument = format!("'{}'", argument.replace("'", "\\'"));
		string_argument
	}

	//I will have to think about how to handle talon captures
	pub fn to_json(&self) -> String {
		let mut result = String::from("{\"name\": \"");
		result.push_str(&self.name);
		result.push_str("\", 'arguments': [\"");
		let mut pushed_first = false;
		for argument in &self.arguments {
			if pushed_first {
				result.push_str("\", \"");
			} else {
				pushed_first = true;
			}
			result.push_str(argument);
		}
		result.push_str("\"]}");
		result
	}

	

	
}