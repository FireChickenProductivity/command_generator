// this is a port of the following python code
// from action_records import BasicAction, read_file_record, TalonCapture, CommandChain, RecordingStart
// import math
// from typing import List
// from text_separation import TextSeparationAnalyzer

// FIVE_MINUTES_IN_SECONDS = 5*60

// def compute_number_of_words(named):
//     return len(named.get_name().split(" "))

// class PotentialCommandInformation:
//     def __init__(self, actions):
//         self.actions = actions
//         self.number_of_times_used: int = 0
//         self.total_number_of_words_dictated: int = 0
//         self.number_of_actions: int = len(self.actions)
//         self.count_repetitions_appropriately_for_number_of_actions()
//         self.chain = None
        
//     def count_repetitions_appropriately_for_number_of_actions(self):
//         for action in self.actions: self.count_repetition_appropriately_for_a_number_of_actions(action)
    
//     def count_repetition_appropriately_for_a_number_of_actions(self, action):
//         argument = action.get_arguments()[0]
//         if action.get_name() == 'repeat' and type(argument) == int:
//             self.number_of_actions += argument - 1
    
//     def get_number_of_actions(self):
//         return len(self.actions)
    
//     def get_average_words_dictated(self):
//         return self.total_number_of_words_dictated/self.number_of_times_used
    
//     def get_number_of_times_used(self):
//         return self.number_of_times_used

//     def get_actions(self):
//         return self.actions
    
//     def is_abstract(self):
//         return False

//     def process_usage(self, command_chain):
//         if self.should_process_usage(command_chain.get_chain_number()):
//             self.process_relevant_usage(command_chain)
    
//     def should_process_usage(self, chain):
//         return self.chain is None or chain > self.chain

//     def process_relevant_usage(self, command_chain):
//         self.number_of_times_used += 1
//         self.chain = command_chain.get_chain_ending_index()
//         self.total_number_of_words_dictated += compute_number_of_words(command_chain)

//     def get_number_of_words_saved(self):
//         return self.get_number_of_times_used()*(self.get_average_words_dictated() - 1)

//     def __repr__(self):
//         return self.__str__()
    
//     def __str__(self):
//         return f'actions: {CommandInformationSet.compute_representation(self)}, number of times used: {self.number_of_times_used}, total number of words dictated: {self.total_number_of_words_dictated}'

// class ActionSequenceSet:
//     def __init__(self):
//         self.set = set()
    
//     def insert(self, actions):
//         representation = compute_string_representation_of_actions(actions)
//         self.set.add(representation)

//     def contains(self, actions):
//         return compute_string_representation_of_actions(actions) in self.set
    
//     def contains_command_actions(self, command):
//         return self.contains(command.get_actions())
    
//     def get_size(self):
//         return len(self.set)

//     def __iter__(self):
//         for sequence in self.set:
//             yield sequence

// class AbstractCommandInstantiation:
//     def __init__(self, command_chain, concrete_command, words_saved):
//         self.command_chain = command_chain
//         self.concrete_command = concrete_command
//         self.words_saved = words_saved

// class PotentialAbstractCommandInformation(PotentialCommandInformation):
//     def __init__(self, instantiation: AbstractCommandInstantiation):
//         self.instantiation_set = ActionSequenceSet()
//         self.number_of_words_saved: int = 0
//         super().__init__(instantiation.command_chain.get_actions())

//     def process_usage(self, instantiation: AbstractCommandInstantiation):
//         if self.should_process_usage(instantiation.command_chain.get_chain_number()):
//             self.instantiation_set.insert(instantiation.concrete_command.get_actions())
//             self.process_relevant_usage(instantiation.command_chain)
//             self.number_of_words_saved += instantiation.words_saved
    
//     def get_number_of_instantiations(self):
//         return self.instantiation_set.get_size()
    
//     def is_abstract(self):
//         return True

//     def get_number_of_words_saved(self):
//         return self.number_of_words_saved

//     def get_instantiation_set(self):
//         return self.instantiation_set

// def compute_repeat_simplified_command_chain(command_chain):
//     new_actions = []
//     last_non_repeat_action = None
//     repeat_count: int = 0
//     for action in command_chain.get_actions():
//         if action == last_non_repeat_action:
//             repeat_count += 1
//         else:
//             if repeat_count > 0:
//                 new_actions.append(BasicAction('repeat', [repeat_count]))
//                 repeat_count = 0
//             new_actions.append(action)
//             last_non_repeat_action = action
//     if repeat_count > 0:
//         new_actions.append(BasicAction('repeat', [repeat_count]))
//     new_command = CommandChain(command_chain.get_name(), new_actions, command_chain.get_chain_number(), command_chain.get_size())
//     return new_command

// def compute_insert_simplified_command_chain(command_chain):
//     new_actions = []
//     current_insert_text = ''
//     for action in command_chain.get_actions():
//         if action.get_name() == 'insert':
//             current_insert_text += action.get_arguments()[0]
//         else:
//             if current_insert_text:
//                 new_actions.append(BasicAction('insert', [current_insert_text]))
//                 current_insert_text = ''
//             new_actions.append(action)
//     if current_insert_text: new_actions.append(BasicAction('insert', [current_insert_text]))
//     new_command = CommandChain(command_chain.get_name(), new_actions, command_chain.get_chain_number(), command_chain.get_size())
//     return new_command

// def compute_string_representation_of_actions(actions):
//     """Adding the string representations of separate lists of actions together should yield the same representation as the union of those lists"""
//     representation = ''
//     for action in actions:
//         representation += action.to_json()
//     return representation

// def should_make_abstract_repeat_representation(command):
//     actions = command.get_actions()
//     if len(actions) <= 2:
//         return False
//     return any(action.get_name() == 'repeat' for action in actions)

// def compute_command_chain_copy_with_new_name_and_actions(command_chain, new_name, new_actions):
//     return CommandChain(new_name, new_actions, command_chain.get_chain_number(), command_chain.get_size())

// def make_abstract_repeat_representation_for(command_chain):
//     actions = command_chain.get_actions()
//     instances = 0
//     new_actions = []
//     new_name = command_chain.get_name()
//     for action in actions:
//         if action.get_name() == 'repeat':
//             instances += 1
//             argument = TalonCapture('number_small', instances, ' - 1')
//             repeat_action = BasicAction('repeat', [argument])
//             new_actions.append(repeat_action)
//             new_name += ' ' + argument.compute_command_component()
//         else:
//             new_actions.append(action)
//     new_command = compute_command_chain_copy_with_new_name_and_actions(command_chain, new_name, new_actions)
//     instantiation = AbstractCommandInstantiation(new_command, command_chain, compute_number_of_words(command_chain) - 2)
//     return instantiation

// def is_prose_inside_inserted_text_with_consistent_separator(prose: str, text: str) -> bool:
//     text_separation_analyzer = TextSeparationAnalyzer(text)
//     text_separation_analyzer.search_for_prose_in_separated_part(prose)
//     text_separation_analyzer.is_prose_separator_consistent()
//     return text_separation_analyzer.has_found_prose()

// class InvalidCaseException(Exception): pass

// def compute_case_string(text: str) -> str:
//     if text.islower(): return 'lower'
//     elif text.isupper(): return 'upper'
//     elif text[0].isupper() and text[1:].islower(): return 'capitalized'
//     else: raise InvalidCaseException()

// def has_valid_case(analyzer: TextSeparationAnalyzer) -> bool:
//     try:
//         for word in analyzer.compute_prose_portion_of_text(): compute_case_string(word)
//         return True
//     except:
//         return False

// def compute_simplified_case_strings_list(case_strings: List) -> List:
//     simplified_case_strings = []
//     new_case_found = False
//     final_case = case_strings[-1]
//     simplified_case_strings.append(final_case)
//     for index in range(len(case_strings) - 2, -1, -1):
//         case = case_strings[index]
//         if case != final_case or new_case_found:
//             simplified_case_strings.append(case)
//             new_case_found = True
//     simplified_case_strings.reverse()
//     return simplified_case_strings

// def compute_case_string_for_prose(analyzer: TextSeparationAnalyzer):
//     prose = analyzer.compute_prose_portion_of_text()
//     case_strings = [compute_case_string(prose_word) for prose_word in prose]
//     simplified_case_strings = compute_simplified_case_strings_list(case_strings)
//     case_string = ' '.join(simplified_case_strings)
//     return case_string

// class ProseMatch:
//     def __init__(self, analyzer: TextSeparationAnalyzer, name: str):
//         self.analyzer = analyzer
//         self.name = name

// def make_abstract_representation_for_prose_command(command_chain, match: ProseMatch, insert_to_modify_index: int):
//     analyzer = match.analyzer
//     actions = command_chain.get_actions()
//     new_actions = actions[:insert_to_modify_index]
//     text_before = analyzer.compute_text_before_prose()
//     if text_before: new_actions.append(BasicAction('insert', [text_before]))
//     prose_argument = TalonCapture('user.text', 1)
//     new_actions.append(BasicAction('user.fire_chicken_auto_generated_command_action_insert_formatted_text', [prose_argument, compute_case_string_for_prose(analyzer), analyzer.get_first_prose_separator()]))
//     text_after = analyzer.compute_text_after_prose()
//     if text_after: new_actions.append(BasicAction('insert', [text_after]))
//     if insert_to_modify_index + 1 < len(actions): new_actions.extend(actions[insert_to_modify_index + 1:])
//     new_command = compute_command_chain_copy_with_new_name_and_actions(command_chain, match.name, new_actions)
//     instantiation = AbstractCommandInstantiation(new_command, command_chain, compute_number_of_words(new_command) - 2)
//     return instantiation

// class InsertAction:
//     def __init__(self, text: str, index: int):
//         self.text = text
//         self.index = index

// def obtain_inserts_from_command_chain(command_chain):
//     return [InsertAction(action.get_arguments()[0], index) for index, action in enumerate(command_chain.get_actions()) if action.get_name() == 'insert']

// def generate_prose_command_command_name(words, starting_index: int, prose_size: int) -> str:
//     command_name_parts = words[:starting_index]
//     command_name_parts.append('<user.text>')
//     command_name_parts.extend(words[starting_index + prose_size:])
//     command_name = ' '.join(command_name_parts)
//     return command_name

// def generate_prose_from_words(words, starting_index: int, prose_size: int) -> str:
//     prose = ' '.join(words[starting_index:starting_index + prose_size])
//     return prose

// def compute_text_analyzer_for_prose_and_insert(prose: str, insert: InsertAction):
//     analyzer = TextSeparationAnalyzer(insert.text)
//     analyzer.search_for_prose_in_separated_part(prose)
//     return analyzer

// class ValidProseNotFoundException(Exception):
//     pass

// def find_prose_match_for_command_given_insert_at_interval(words, insert, starting_index, prose_size):
//     prose = generate_prose_from_words(words, starting_index, prose_size)
//     analyzer = compute_text_analyzer_for_prose_and_insert(prose, insert)
//     if analyzer.is_prose_separator_consistent() and analyzer.has_found_prose() and has_valid_case(analyzer):
//         command_name = generate_prose_command_command_name(words, starting_index, prose_size)
//         return ProseMatch(analyzer, command_name)
//     raise ValidProseNotFoundException()

// def find_prose_matches_for_command_given_insert_at_starting_index(words, insert, starting_index, max_prose_size_to_consider):
//     matches = []
//     maximum_size = min(max_prose_size_to_consider, len(words) - starting_index + 1)
//     for prose_size in range(1, maximum_size):
//         try: matches.append(find_prose_match_for_command_given_insert_at_interval(words, insert, starting_index, prose_size))
//         except ValidProseNotFoundException: break
//     return matches

// def find_prose_matches_for_command_given_insert(command_chain, insert, max_prose_size_to_consider):
//     dictation: str = command_chain.get_name()
//     words = dictation.split(' ')
//     matches = []
//     for starting_index in range(len(words)): matches.extend(find_prose_matches_for_command_given_insert_at_starting_index(words, insert, starting_index, max_prose_size_to_consider))
//     return matches

// def is_acceptable_abstract_representation(representation):
//     return len(representation.get_actions()) > 1

// def make_abstract_prose_representations_for_command_given_insert(command_chain, insert, max_prose_size_to_consider):
//     abstract_representations = []
//     prose_matches = find_prose_matches_for_command_given_insert(command_chain, insert, max_prose_size_to_consider)
//     for match in prose_matches:
//         abstract_representation = make_abstract_representation_for_prose_command(command_chain, match, insert.index)
//         if is_acceptable_abstract_representation(abstract_representation.command_chain):
//             abstract_representations.append(abstract_representation)
//     return abstract_representations

// def make_abstract_prose_representations_for_command_given_inserts(command_chain, inserts, max_prose_size_to_consider):
//     abstract_representations = []
//     for insert in inserts: 
//         representations_given_insert = make_abstract_prose_representations_for_command_given_insert(command_chain, insert, max_prose_size_to_consider)
//         abstract_representations.extend(representations_given_insert)
//     return abstract_representations

// def make_abstract_prose_representations_for_command(command_chain, max_prose_size_to_consider = 10):
//     inserts = obtain_inserts_from_command_chain(command_chain)
//     if len(inserts) == 0: return []
//     else: return make_abstract_prose_representations_for_command_given_inserts(command_chain, inserts, max_prose_size_to_consider)

// def basic_command_filter(command: PotentialCommandInformation):
//     return command.get_average_words_dictated() >= 2 and command.get_number_of_times_used() > 1 and \
//             (not command.is_abstract() or command.get_number_of_instantiations() > 2 and command.get_average_words_dictated() > 2) and \
//             (command.get_number_of_actions()/command.get_average_words_dictated() < 2 or \
//             command.get_number_of_actions()*math.sqrt(command.get_number_of_times_used()) > command.get_average_words_dictated())

// def is_record_entry_recording_start(record_entry) -> bool:
//     return type(record_entry) == RecordingStart

// def is_command_after_chain_start_exceeding_time_gap_threshold(record_entry, chain_start_index, current_chain_index) -> bool:
//     return current_chain_index > chain_start_index and record_entry.is_command_record() and record_entry.is_time_information_available() \
//     and record_entry.get_seconds_since_action() > FIVE_MINUTES_IN_SECONDS

// def should_command_chain_not_cross_entry_at_record_index(record, chain_start_index, current_chain_index) -> bool:
//     record_entry = record[current_chain_index]
//     return is_record_entry_recording_start(record_entry) or \
//         is_command_after_chain_start_exceeding_time_gap_threshold(record_entry, chain_start_index, current_chain_index)

// worker_record=None
// def initialize_worker_with_record(record):
//     global worker_record
//     worker_record = record

// def add_next_record_command_to_chain(record, command_chain):
//     command_chain.append_command(record[command_chain.get_next_chain_index()])

// def simplify_command_chain(command_chain):
//     simplified_command_chain = compute_insert_simplified_command_chain(command_chain)
//     simplified_command_chain = compute_repeat_simplified_command_chain(simplified_command_chain)
//     return simplified_command_chain

// def do_chain_asynchronous_work(start_index, ending_index):
//     concrete_chain = CommandChain(None, [], start_index)
//     for _ in range(start_index, ending_index+1):
//         add_next_record_command_to_chain(worker_record, concrete_chain)
//     simplified_command_chain = simplify_command_chain(concrete_chain)
//     abstract_commands = create_abstract_commands(simplified_command_chain)
//     abstract_representations = [CommandInformationSet.compute_representation(a.command_chain) for a in abstract_commands]
//     concrete_representation = CommandInformationSet.compute_representation(simplified_command_chain)
//     return simplified_command_chain, concrete_representation, abstract_commands, abstract_representations
    
// def create_abstract_commands(command_chain):
//     commands = []
//     if should_make_abstract_repeat_representation(command_chain):
//         abstract_repeat_representation = make_abstract_repeat_representation_for(command_chain)
//         commands.append(abstract_repeat_representation)
//     abstract_prose_commands = make_abstract_prose_representations_for_command(command_chain)
//     commands.extend(abstract_prose_commands)
//     return commands

// def compute_chain_size(record, chain, chain_target):
//     num_targets = 0
//     for chain_ending_index in range(chain, chain_target): 
//         if should_command_chain_not_cross_entry_at_record_index(record, chain, chain_ending_index): break
//         num_targets += 1
//     return num_targets

// class CommandInformationSet:
//     def __init__(self):
//         self.commands = {}

//     def insert_command(self, command, representation):
//         self.commands[representation] = command
    
//     def process_abstract_command_usage(self, instantiation: AbstractCommandInstantiation, representation: str=None):
//         if not representation:
//             representation = CommandInformationSet.compute_representation(instantiation.command_chain)
//         if representation not in self.commands:
//             self.insert_command(PotentialAbstractCommandInformation(instantiation), representation)
//         self.commands[representation].process_usage(instantiation)
    
//     def handle_needed_abstract_commands(self, command_chain):
//         abstract_commands = create_abstract_commands(command_chain)
//         for abstract_command in abstract_commands: self.process_abstract_command_usage(abstract_command)

//     def process_concrete_command_usage(self, command_chain, representation: str=None):
//         if not representation:
//             representation = CommandInformationSet.compute_representation(command_chain)
//         if representation not in self.commands:
//             self.insert_command(PotentialCommandInformation(command_chain.get_actions()), representation)
//         self.commands[representation].process_usage(command_chain)

//     def process_command_usage(self, command_chain):
//         self.process_concrete_command_usage(command_chain)
//         self.handle_needed_abstract_commands(command_chain)
    
//     def process_partial_chain_usage(self, record, command_chain):
//         add_next_record_command_to_chain(record, command_chain)
//         simplified_command_chain = simplify_command_chain(command_chain)
//         self.process_command_usage(simplified_command_chain)

//     def process_chain_usage_in_parallel(self, chain, num_targets, pool):
//         results = []
//         for chain_ending_index in range(chain, chain + num_targets):
//             result = pool.apply_async(do_chain_asynchronous_work, (chain, chain_ending_index))
//             results.append(result)
//         for result in results:
//             concrete_chain, concrete_representation, abstract_commands, abstract_representations = result.get()
//             self.process_concrete_command_usage(concrete_chain, concrete_representation)
//             for i in range(len(abstract_commands)):
//                 self.process_abstract_command_usage(abstract_commands[i], abstract_representations[i])

//     def process_chain_usage_sequentially(self, record, chain, chain_target):
//         command_chain: CommandChain = CommandChain(None, [], chain)
//         for chain_ending_index in range(chain, chain_target): 
//             if should_command_chain_not_cross_entry_at_record_index(record, chain, chain_ending_index): break
//             self.process_partial_chain_usage(record, command_chain)

//     def process_chain_usage(self, record, chain, max_command_chain_considered, verbose = False, pool = None):
//         chain_target = min(len(record), chain + max_command_chain_considered)
//         should_run_sequentially = True
//         if pool is not None:
//             num_targets = compute_chain_size(record, chain, chain_target)
//             if num_targets > 1:
//                 self.process_chain_usage_in_parallel(chain, num_targets, pool)
//                 should_run_sequentially = False
//         if should_run_sequentially:
//             self.process_chain_usage_sequentially(record, chain, chain_target)
//         if verbose: print('chain', chain + 1, 'out of', len(record), 'target: ', chain_target)

//     @staticmethod
//     def compute_representation(command):
//         actions = command.get_actions()
//         representation = compute_string_representation_of_actions(actions)
//         return representation
    
//     def get_commands_meeting_condition(self, condition):
//         commands_to_output = [command for command in self.commands.values() if condition(command)]
//         return commands_to_output
    
//     def contains_command_with_representation(self, representation: str):
//         return representation in self.commands
    
//     def contains_command(self, command):
//         representation = CommandInformationSet.compute_representation(command)
//         return self.contains_command_with_representation(representation)

//     def get_size(self):
//         return len(self.commands)

//     def __repr__(self):
//         return self.__str__()
    
//     def __str__(self):
//         representation: str = ''
//         for command in self.commands.values():
//             representation += str(command) + '\n'
//         return representation

const FIVE_MINUTES_IN_SECONDS: u32 = 5 * 60;
const DEFAULT_MAX_PROSE_SIZE_TO_CONSIDER: u32 = 10;
use crate::action_records::{
	Argument,
	BasicAction,
	Command,
	CommandChain,
	TalonCapture,
	Entry
};
use crate::action_utilities::*;
use crate::text_separation::{
	TextSeparationAnalyzer,
	compute_case_string_for_prose,
	has_valid_case,
};
use std::collections::HashSet;

fn compute_number_of_words(command_chain: &CommandChain) -> u32 {
	command_chain.get_command().get_name().split_whitespace().count() as u32
}


pub struct PotentialCommandInformation {
	actions: Vec<BasicAction>,
	number_of_times_used: u32,
	total_number_of_words_dictated: u32,
	number_of_actions: usize,
	chain: Option<u32>,
}

fn compute_number_of_actions(actions: &Vec<BasicAction>) -> usize {
	let mut number_of_actions = actions.len();
	for action in actions {
		if action.get_name() == "repeat" {
			if let Some(argument) = action.get_arguments().first() {
				if let Argument::IntArgument(repeat_count) = argument {
					let unsigned_repeat_count = *repeat_count as usize;
					number_of_actions += unsigned_repeat_count - 1;
				}
			}
		}
	}
	number_of_actions
}

impl PotentialCommandInformation {
	pub fn new(actions: Vec<BasicAction>) -> Self {
		PotentialCommandInformation {
			actions,
			number_of_times_used: 0,
			total_number_of_words_dictated: 0,
			number_of_actions: 0,
			chain: None,
		}
	}

	pub fn get_number_of_actions(&self) -> usize {
		self.number_of_actions
	}

	pub fn get_average_words_dictated(&self) -> f32 {
		if self.number_of_times_used == 0 {
			return 0.0;
		}
		self.total_number_of_words_dictated as f32 / self.number_of_times_used as f32
	}

	pub fn get_number_of_times_used(&self) -> u32 {
		self.number_of_times_used
	}

	pub fn get_actions(&self) -> &Vec<BasicAction> {
		&self.actions
	}

	pub fn process_usage(&mut self, command_chain: &CommandChain) {
		if self.should_process_usage(command_chain.get_chain_number()) {
			self.process_relevant_usage(command_chain);
		}
	}

	fn should_process_usage(&self, chain: u32) -> bool {
		match self.chain {
			Some(existing_chain) => existing_chain < chain,
			None => true,
		}
	}

	fn process_relevant_usage(&mut self, command_chain: &CommandChain) {
		self.number_of_times_used += 1;
		self.chain = Some(command_chain.get_chain_ending_index());
		self.total_number_of_words_dictated += compute_number_of_words(command_chain);
	}

	pub fn get_number_of_words_saved(&self) -> u32 {
		self.get_number_of_times_used() * (self.get_average_words_dictated() as u32 - 1)
	}
}

pub struct ActionSet {
	set: HashSet<String>,
}

pub fn compute_string_representation_of_actions(actions: &Vec<BasicAction>) -> String {
	actions.iter().map(|action| action.to_json()).collect::<Vec<String>>().join("")
}

impl ActionSet {
	pub fn new() -> Self {
		Self {
			set: HashSet::new(),
		}
	}

	pub fn insert(&mut self, actions: &Vec<BasicAction>) {
		let representation = compute_string_representation_of_actions(actions);
		self.set.insert(representation);
	}

	pub fn contains(&self, actions: &Vec<BasicAction>) -> bool {
		let representation = compute_string_representation_of_actions(actions);
		self.set.contains(&representation)
	}

	pub fn get_size(&self) -> usize {
		self.set.len()
	}
}

pub struct AbstractCommandInstantiation {
	pub command_chain: CommandChain,
	pub concrete_command: CommandChain,
	pub words_saved: u32,
}

pub struct PotentialAbstractCommandInformation {
	instantiation_set: ActionSet,
	number_of_words_saved: u32,
	info: PotentialCommandInformation,
}

impl PotentialAbstractCommandInformation {
	pub fn new(instantiation: AbstractCommandInstantiation) -> Self {
		let actions = instantiation.command_chain.get_command().get_actions();
		let potential_command_information = PotentialCommandInformation::new(actions.clone());
		Self {
			instantiation_set: ActionSet::new(),
			number_of_words_saved: instantiation.words_saved,
			info: potential_command_information,
		}
	}

	pub fn process_usage(&mut self, instantiation: AbstractCommandInstantiation) {
		let chain_number = instantiation.command_chain.get_chain_number();
		if self.info.should_process_usage(chain_number) {
			let actions = instantiation.concrete_command.get_command().get_actions();
			self.instantiation_set.insert(&actions);
			self.info.process_relevant_usage(&instantiation.command_chain);
			self.number_of_words_saved += instantiation.words_saved;
		}
	}

	pub fn get_number_of_instantiations(&self) -> usize {
		self.instantiation_set.get_size()
	}

	pub fn get_number_of_words_saved(&self) -> u32 {
		self.number_of_words_saved
	}

	pub fn get_instantiation_set(&self) -> &ActionSet {
		&self.instantiation_set
	}

	pub fn get_potential_command_information(&self) -> &PotentialCommandInformation {
		&self.info
	}
}

enum Information {
	Concrete(PotentialCommandInformation),
	Abstract(PotentialAbstractCommandInformation),
}

fn create_repeat_action(repeat_count: i32) -> BasicAction {
	BasicAction::new("repeat", vec![Argument::IntArgument(repeat_count)])
}

fn compute_command_chain_copy_with_new_name_and_actions(
	command_chain: &CommandChain,
	new_name: &str,
	new_actions: Vec<BasicAction>,
) -> CommandChain {
	let new_command = Command::new(new_name, new_actions, command_chain.get_command().get_seconds_since_last_action());
	CommandChain::new(new_command, command_chain.get_chain_number(), command_chain.get_size())
}

fn compute_repeat_simplified_command_chain(command_chain: &CommandChain) -> CommandChain {
	let mut new_actions = Vec::new();
	let mut last_non_repeat_action: Option<BasicAction> = None;
	let mut repeat_count: i32 = 0;

	for action in command_chain.get_command().get_actions() {

		if last_non_repeat_action.is_some() && action == last_non_repeat_action.as_ref().unwrap() {
			repeat_count += 1;
		} else {
			if repeat_count > 0 {
				new_actions.push(create_repeat_action(repeat_count));
				repeat_count = 0;
			}
			new_actions.push(action.clone());
			last_non_repeat_action = Some(action.clone());
		}
	}

	if repeat_count > 0 {
		new_actions.push(create_repeat_action(repeat_count));
	}

	let new_command = Command::new(command_chain.get_command().get_name(), new_actions, command_chain.get_command().get_seconds_since_last_action());
	CommandChain::new(new_command, command_chain.get_chain_number(), command_chain.get_size())
}

fn compute_insert_simplified_command_chain(command_chain: &CommandChain) -> CommandChain {
	let mut new_actions = Vec::new();
	let mut current_insert_text = String::new();

	for action in command_chain.get_command().get_actions() {
		if is_insert(action) {
			current_insert_text += get_insert_text(action);
		} else {
			if !current_insert_text.is_empty() {
				new_actions.push(create_insert_action(current_insert_text.as_str()));
				current_insert_text.clear();
			}
			new_actions.push(action.clone());
		}
	}

	if !current_insert_text.is_empty() {
		new_actions.push(create_insert_action(current_insert_text.as_str()));
	}

	let new_command = Command::new(command_chain.get_command().get_name(), new_actions, command_chain.get_command().get_seconds_since_last_action());
	CommandChain::new(new_command, command_chain.get_chain_number(), command_chain.get_size())
}

fn should_make_abstract_repeat_representation(command_chain: &CommandChain) -> bool {
	let actions = command_chain.get_command().get_actions();
	if actions.len() <= 2 {
		return false;
	}
	actions.iter().any(|action| action.get_name() == "repeat")
}

fn make_abstract_repeat_representation_for(command_chain: &CommandChain) -> AbstractCommandInstantiation {
	let actions = command_chain.get_command().get_actions();
	let mut instances = 0;
	let mut new_actions = Vec::new();
	let mut new_name = command_chain.get_command().get_name().to_string();

	for action in actions {
		if action.get_name() == "repeat" {
			instances += 1;
			let mut capture = TalonCapture::new("number_small", instances);
			capture.set_postfix(" - 1");
			new_name.push_str(&format!(" {}", capture.compute_command_component()));
			let argument = Argument::CaptureArgument(capture);
			let repeat_action = BasicAction::new("repeat", vec![argument]);
			new_actions.push(repeat_action);
		} else {
			new_actions.push(action.clone());
		}
	}

	let new_command = compute_command_chain_copy_with_new_name_and_actions(command_chain, &new_name, new_actions);
	let words_saved = compute_number_of_words(command_chain) - 2; 
	AbstractCommandInstantiation {
		command_chain: new_command,
		concrete_command: command_chain.clone(),
		words_saved: words_saved as u32,
	}
}

fn is_prose_inside_text_with_consistent_separator(prose: &str, text: &str) -> bool {
	let mut text_separation_analyzer = TextSeparationAnalyzer::new_from_text(text);
	text_separation_analyzer.search_for_prose_in_separated_part(prose);
	text_separation_analyzer.is_prose_separator_consistent();
	text_separation_analyzer.has_found_prose()
}

struct ProseMatch {
	analyzer: TextSeparationAnalyzer,
	name: String,
}

fn make_abstract_representation_for_prose_command(command_chain: &CommandChain, prose_match: &ProseMatch, insert_to_modify_index: usize) -> AbstractCommandInstantiation {
	let analyzer = &prose_match.analyzer;
	let actions = command_chain.get_command().get_actions();
	let mut new_actions = actions[..insert_to_modify_index].to_vec();
	
	{
		let text_before = analyzer.compute_text_before_prose();
		if !text_before.is_empty() {
			new_actions.push(create_insert_action(&text_before));
		}
	}
	
	let prose_argument = TalonCapture::new("user.text", 1);
	new_actions.push(BasicAction::new(
		"user.fire_chicken_auto_generated_command_action_insert_formatted_text",
		vec![
			Argument::CaptureArgument(prose_argument), Argument::StringArgument(compute_case_string_for_prose(&analyzer)), Argument::StringArgument(analyzer.get_first_prose_separator())
		],
	));
	
	{
		let text_after = analyzer.compute_text_after_prose();
		if !text_after.is_empty() {
			new_actions.push(create_insert_action(&text_after));
		}
	}
	
	if insert_to_modify_index + 1 < actions.len() {
		new_actions.extend_from_slice(&actions[insert_to_modify_index + 1..]);
	}
	
	let new_command = compute_command_chain_copy_with_new_name_and_actions(command_chain, &prose_match.name, new_actions);
	let words_saved = compute_number_of_words(&new_command) - 2;
	AbstractCommandInstantiation {
		command_chain: new_command,
		concrete_command: command_chain.clone(),
		words_saved: words_saved as u32,
	}
}

struct InsertAction {
	text: String,
	index: usize,
}

fn obtain_inserts_from_command_chain(command_chain: &CommandChain) -> Vec<InsertAction> {
	command_chain.get_command()
		.get_actions()
		.iter()
		.enumerate()
		.filter_map(|(index, action)| {
			if is_insert(action) {
				let insert_text = get_insert_text(action);
				Some(InsertAction {
					text: insert_text.to_string(),
					index,
				})
			} else {
				None
			}
		})
		.collect()
}

fn generate_prose_command_name(words: &[&str], starting_index: usize, prose_size: usize) -> String {
	let mut command_name_parts = words[..starting_index].to_vec();
	command_name_parts.push("<user.text>");
	command_name_parts.extend_from_slice(&words[starting_index + prose_size..]);
	command_name_parts.join(" ")
}

fn generate_prose_from_words(words: &[&str], starting_index: usize, prose_size: usize) -> String {
	words[starting_index..starting_index + prose_size].join(" ")
}

fn compute_text_analyzer_for_prose_and_insert(prose: &str, insert: &InsertAction) -> TextSeparationAnalyzer {
	let mut analyzer = TextSeparationAnalyzer::new_from_text(&insert.text);
	analyzer.search_for_prose_in_separated_part(prose);
	analyzer
}

fn find_prose_match_for_command_given_insert_at_interval(
	words: &[&str],
	insert: &InsertAction,
	starting_index: usize,
	prose_size: usize,
) -> Result<ProseMatch, ()> {
	let prose = generate_prose_from_words(words, starting_index, prose_size);
	let analyzer = compute_text_analyzer_for_prose_and_insert(&prose, insert);
	if analyzer.is_prose_separator_consistent() && analyzer.has_found_prose() && has_valid_case(&analyzer) {
		let command_name = generate_prose_command_name(words, starting_index, prose_size);
		Ok(ProseMatch {
			analyzer,
			name: command_name,
		})
	} else {
		Err(())
	}
}

fn find_prose_matches_for_command_given_insert_at_starting_index(
	words: &[&str],
	insert: &InsertAction,
	starting_index: usize,
	max_prose_size_to_consider: usize,
) -> Vec<ProseMatch> {
	let mut matches = Vec::new();
	let maximum_size = max_prose_size_to_consider.min(words.len() - starting_index + 1);
	for prose_size in 1..maximum_size {
		if let Ok(match_found) = find_prose_match_for_command_given_insert_at_interval(words, insert, starting_index, prose_size) {
			matches.push(match_found);
		} else {
			break;
		}
	}
	matches
}

fn find_prose_matches_for_command_given_insert(
	command_chain: &CommandChain,
	insert: &InsertAction,
	max_prose_size_to_consider: usize,
) -> Vec<ProseMatch> {
	let dictation = command_chain.get_command().get_name();
	let words: Vec<&str> = dictation.split_whitespace().collect();
	let mut matches = Vec::new();
	for starting_index in 0..words.len() {
		matches.extend(find_prose_matches_for_command_given_insert_at_starting_index(
			&words, insert, starting_index, max_prose_size_to_consider,
		));
	}
	matches
}

fn is_acceptable_abstract_representation(representation: &CommandChain) -> bool {
	representation.get_command().get_actions().len() > 1
}

fn make_abstract_prose_representations_for_command_given_insert(
	command_chain: &CommandChain,
	insert: &InsertAction,
	max_prose_size_to_consider: usize,
) -> Vec<AbstractCommandInstantiation> {
	let mut abstract_representations = Vec::new();
	let prose_matches = find_prose_matches_for_command_given_insert(command_chain, insert, max_prose_size_to_consider);
	for match_found in prose_matches {
		let abstract_representation = make_abstract_representation_for_prose_command(&command_chain, &match_found, insert.index);
		if is_acceptable_abstract_representation(&abstract_representation.command_chain) {
			abstract_representations.push(abstract_representation);
		}
	}
	abstract_representations
}

fn make_abstract_prose_representations_for_command_given_inserts(
	command_chain: &CommandChain,
	inserts: &[InsertAction],
	max_prose_size_to_consider: usize,
) -> Vec<AbstractCommandInstantiation> {
	let mut abstract_representations = Vec::new();
	for insert in inserts {
		let representations_given_insert = make_abstract_prose_representations_for_command_given_insert(command_chain, insert, max_prose_size_to_consider);
		abstract_representations.extend(representations_given_insert);
	}
	abstract_representations
}

pub fn make_abstract_prose_representations_for_command(
	command_chain: &CommandChain,
	max_prose_size_to_consider: usize,
) -> Vec<AbstractCommandInstantiation> {
	let inserts = obtain_inserts_from_command_chain(command_chain);
	if inserts.is_empty() {
		Vec::new()
	} else {
		make_abstract_prose_representations_for_command_given_inserts(command_chain, &inserts, max_prose_size_to_consider)
	}
}

fn basic_command_filter(info: &Information) -> bool {
	if let Information::Abstract(abstract_info) = info {
		if abstract_info.get_potential_command_information().get_average_words_dictated() < 2.0 || abstract_info.get_number_of_instantiations() <= 2{
			return false;
			
		}
	}
	let concrete = match info {
		Information::Concrete(concrete_info) => concrete_info,
		Information::Abstract(abstract_info) => &abstract_info.get_potential_command_information(),
	};
	concrete.get_number_of_actions() as f32 / concrete.get_average_words_dictated() < 2.0 ||
		concrete.get_number_of_actions() as f32 * (concrete.get_number_of_times_used() as f32).sqrt() > concrete.get_average_words_dictated()
}

fn is_command_after_chain_start_exceeding_time_gap_threshold(
	record_entry: &Entry,
	chain_start_index: usize,
	current_chain_index: usize,
) -> bool {
	match record_entry {
		Entry::RecordingStart => true,
		Entry::Command(record_entry) => {
			match record_entry.get_seconds_since_last_action() {
				Some(seconds) => current_chain_index > chain_start_index && 
										seconds > FIVE_MINUTES_IN_SECONDS,
				None => false,
			}
		}
	}
}

