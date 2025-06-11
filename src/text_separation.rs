// This module is a rust port of the following python code

// from typing import List

// def is_character_alpha(character: str):
//     return character.isalpha()

// class TextSeparation:
//     def __init__(self, string: str, character_filter):
//         self.separated_parts = []
//         self.separators = []
//         self.current_separated_part = ''
//         self.current_separator = ''
//         self.text_prefix = ''
//         for character in string: self._process_character(character, character_filter)
//         if not self.current_separated_part: self._handle_separator()
//         if not self.current_separator: self._add_separated_part()
    
//     def _process_character(self, character, character_filter):
//         if character_filter(character): 
//             if not self.current_separated_part: self._handle_separator()
//             self.current_separated_part += character
//         else:
//             if not self.current_separator and self.current_separated_part: self._add_separated_part()
//             self.current_separator += character

//     def _handle_separator(self):
//         if len(self.separated_parts) > 0:
//             self.separators.append(self.current_separator)
//         else:
//             self.text_prefix = self.current_separator
//         self.current_separator = ''
    
//     def _add_separated_part(self):
//         self.separated_parts.append(self.current_separated_part)
//         self.current_separated_part = ''
    
//     def get_separated_parts(self):
//         return self.separated_parts

//     def get_separators(self):
//         return self.separators
    
//     def get_prefix(self):
//         return self.text_prefix

// class TextSeparationAnalyzer:
//     def __init__(self, text: str, character_filter = is_character_alpha):
//         self.text_separation = TextSeparation(text, character_filter)
//         self.prose_index = None
//         self.final_prose_index_into_separated_parts = None
//         self.prose_beginning_index = None
//         self.prose_ending_index = None
//         self.number_of_prose_words = None
//         self.found_prose: bool = False
//         self.prose: str = None

//     def search_for_prose_beginning_at_separated_part_index(self, words, separated_parts, index):
//         initial_separated_part = separated_parts[index].lower()
//         first_word = words[0]
//         if initial_separated_part.endswith(first_word): self.prose_beginning_index = initial_separated_part.rfind(first_word)
    
//     def search_for_prose_at_separated_part_index_beginning(self, prose_without_spaces, separated_parts, index):
//         if prose_without_spaces in separated_parts[index].lower(): 
//             self.prose_beginning_index = separated_parts[index].lower().find(prose_without_spaces)
//             self.prose_ending_index = len(prose_without_spaces) + self.prose_beginning_index
//             self.found_prose = True
//             self.final_prose_index_into_separated_parts = index
    
//     def is_prose_middle_different_from_separated_parts_at_index(self, words, separated_parts, index):
//         for prose_index in range(1, len(words) - 1):
//             word = words[prose_index]
//             separated_part: str = separated_parts[prose_index + index].lower()
//             if separated_part != word: return True
//         return False
    
//     def perform_final_prose_search_at_index(self, words, separated_parts, index):
//         if len(words) == 1:
//             self.found_prose = True
//             self.final_prose_index_into_separated_parts = self.prose_index
//         else:
//             self.final_prose_index_into_separated_parts = index + len(words) - 1
//             final_separated_part = separated_parts[self.final_prose_index_into_separated_parts].lower()
//             last_word = words[-1]
//             if final_separated_part.startswith(last_word): 
//                 self.prose_ending_index = len(last_word)
//                 self.found_prose = True
    
//     def reset_indices(self):
//         self.prose_beginning_index = None
//         self.prose_index = None
//         self.prose_ending_index = None
//         self.final_prose_index_into_separated_parts = None

//     def search_for_prose_at_separated_part_index(self, prose_without_spaces: str, words, index: int):
//         self.reset_indices()
        
//         separated_parts = self.text_separation.get_separated_parts()

//         self.search_for_prose_at_separated_part_index_beginning(prose_without_spaces, separated_parts, index)
//         if self.found_prose: return
//         if len(words) + index > len(separated_parts): return

//         self.search_for_prose_beginning_at_separated_part_index(words, separated_parts, index)
//         if self.prose_beginning_index is None: return

//         if self.is_prose_middle_different_from_separated_parts_at_index(words, separated_parts, index): return 

//         self.perform_final_prose_search_at_index(words, separated_parts, index)
    
//     def search_for_prose_in_separated_part(self, prose: str):
//         lowercase_prose = prose.lower()
//         prose_without_spaces = lowercase_prose.replace(' ', '')
//         words = lowercase_prose.split(' ')
//         self.number_of_prose_words = len(words)
//         self.prose = prose
//         for index in range(len(self.text_separation.get_separated_parts())):
//             self.search_for_prose_at_separated_part_index(prose_without_spaces, words, index)
//             self.prose_index = index
//             if self.found_prose: return
//         self.found_prose = False
//         return
    
//     def is_separator_consistent(self, starting_index: int = 0, ending_index: int = -1):
//         separators = self.text_separation.get_separators()[starting_index:ending_index]
//         if len(separators) <= 1: return True
//         initial_separator = separators[0]
//         for index in range(1, len(separators)):
//             if separators[index] != initial_separator: return False
//         return True

//     def get_prose_index(self):
//         return self.prose_index
    
//     def get_prose_beginning_index(self):
//         return self.prose_beginning_index
    
//     def get_prose_ending_index(self):
//         return self.prose_ending_index

//     def is_prose_separator_consistent(self):
//         return self.is_separator_consistent(self.prose_index, self.final_prose_index_into_separated_parts)

//     def get_first_prose_separator(self) -> str:
//         separators = self.text_separation.get_separators()
//         if self.prose_index < len(separators) and self.prose_index != self.final_prose_index_into_separated_parts: return separators[self.prose_index]
//         else: return ''

//     def has_found_prose(self):
//         return self.found_prose
    
//     def compute_text_before_prose(self) -> str:
//         text: str = self.text_separation.get_prefix()
//         separated_parts = self.text_separation.get_separated_parts()
//         separators = self.text_separation.get_separators()
//         for index in range(self.prose_index):
//             text += separated_parts[index]
//             text += separators[index]
//         text += separated_parts[self.prose_index][0:self.prose_beginning_index]
//         return text
    
//     def compute_text_after_prose(self) -> str:
//         separated_parts = self.text_separation.get_separated_parts()
//         separators = self.text_separation.get_separators()
//         text: str = ''
//         first_word: str = separated_parts[self.final_prose_index_into_separated_parts]
//         if self.prose_ending_index < len(first_word): text += first_word[self.prose_ending_index:]
//         if self.final_prose_index_into_separated_parts < len(separators): text += separators[self.final_prose_index_into_separated_parts]
//         for index in range(self.final_prose_index_into_separated_parts + 1, len(separated_parts)):
//             text += separated_parts[index]
//             if index < len(separators): text += separators[index]
//         return text
    
//     def _compute_prose_portion_of_nonseparated_text(self):
//         words = self.prose.split(' ')
//         prose_portion_of_text_as_string = self.text_separation.get_separated_parts()[self.prose_index][self.prose_beginning_index:self.prose_ending_index]
//         word_starting_index = 0
//         words_from_text = []
//         for word in words:
//             word_ending_index = word_starting_index + len(word)
//             word_from_text = prose_portion_of_text_as_string[word_starting_index:word_ending_index]
//             words_from_text.append(word_from_text)
//             word_starting_index = word_ending_index
//         return words_from_text

//     def _compute_prose_portion_of_separated_text(self, prose_final_index):
//         prose_words = []
//         separated_parts = self.text_separation.get_separated_parts()
//         prose_words.append(separated_parts[self.prose_index][self.prose_beginning_index:])
//         prose_words.extend([separated_parts[index] for index in range(self.prose_index + 1, prose_final_index)])
//         prose_words.append(separated_parts[prose_final_index][:self.prose_ending_index])
//         return prose_words

//     def compute_prose_portion_of_text(self) -> List[str]:
//         if self.prose_index == self.final_prose_index_into_separated_parts: return self._compute_prose_portion_of_nonseparated_text()
//         else: return self._compute_prose_portion_of_separated_text(self.final_prose_index_into_separated_parts)

use crate::action_records::{BasicAction, Argument};

pub fn is_character_alpha(character: char) -> bool {
	character.is_alphabetic()
}

pub struct TextSeparation {
	separated_parts: Vec<String>,
	separators: Vec<String>,
	current_separated_part: String,
	current_separator: String,
	text_prefix: String,
}

impl TextSeparation {
	pub fn new(string: &str, character_filter: fn(char) -> bool) -> Self {
		let mut instance = TextSeparation {
			separated_parts: Vec::new(),
			separators: Vec::new(),
			current_separated_part: String::new(),
			current_separator: String::new(),
			text_prefix: String::new(),
		};
		string.chars().for_each(| character | {
			instance.process_character(character, character_filter);
		});

		if instance.current_separated_part.is_empty() {
			instance.handle_separator();
		}
		if instance.current_separator.is_empty() {
			instance.add_separated_part();
		}
		instance
	}

	fn process_character(&mut self, character: char, character_filter: fn(char) -> bool) {
		if character_filter(character) {
			if self.current_separated_part.is_empty() {
				self.handle_separator();
			}
			self.current_separated_part.push(character);
		} else {
			if self.current_separator.is_empty() && !self.current_separated_part.is_empty() {
				self.add_separated_part();
			}
			self.current_separator.push(character);
		}
	}

	fn handle_separator(&mut self) {
		if !self.separated_parts.is_empty() {
			self.separators.push(self.current_separator.clone());
		} else {
			self.text_prefix = self.current_separator.clone();
		}
		self.current_separator.clear();
	}

	fn add_separated_part(&mut self) {
		self.separated_parts.push(self.current_separated_part.clone());
		self.current_separated_part.clear();
	}

	pub fn get_separated_parts(&self) -> &Vec<String> {
		&self.separated_parts
	}

	pub fn get_separators(&self) -> &Vec<String> {
		&self.separators
	}

	pub fn get_prefix(&self) -> &String {
		&self.text_prefix
	}
}

struct TextSeparationAnalyzer {
	text_separation: TextSeparation,
	prose_index: Option<usize>,
	final_prose_index_into_separated_parts: Option<usize>,
	prose_beginning_index: Option<usize>,
	prose_ending_index: Option<usize>,
	number_of_prose_words: Option<usize>,
	found_prose: bool,
	prose: Option<String>,
}

impl TextSeparationAnalyzer {
	pub fn new(text: &str, character_filter: fn(char) -> bool) -> Self {
		TextSeparationAnalyzer {
			text_separation: TextSeparation::new(text, character_filter),
			prose_index: None,
			final_prose_index_into_separated_parts: None,
			prose_beginning_index: None,
			prose_ending_index: None,
			number_of_prose_words: None,
			found_prose: false,
			prose: None,
		}
	}
	
	/// Assumes that initial_separated_part is lowercase
	fn search_for_prose_beginning_at_separated_part_index(&mut self, words: &[String], initial_separated_part: &str) {
		let first_word = &words[0];
		if initial_separated_part.ends_with(first_word) {
			self.prose_beginning_index = Some(initial_separated_part.rfind(first_word).unwrap());
		}
	}
	
	fn search_for_prose_at_separated_part_index_beginning(&mut self, prose_without_spaces: &str, lowercase_part: &str, index: usize) {
		if let Some(prose_beginning_index) = lowercase_part.find(prose_without_spaces) {
			self.prose_beginning_index = Some(prose_beginning_index);
			self.prose_ending_index = Some(self.prose_beginning_index.unwrap() + prose_without_spaces.len());
			self.found_prose = true;
			self.final_prose_index_into_separated_parts = Some(index);
		}
	}

	fn is_prose_middle_different_from_separated_parts_at_index(&self, words: &[String], separated_parts: &[String], index: usize) -> bool {
		for prose_index in 1..words.len() - 1 {
			let word = &words[prose_index];
			let separated_part = &separated_parts[prose_index + index].to_lowercase();
			if separated_part != word {
				return true;
			}
		}
		false
	}
	fn perform_final_prose_search_at_index(&mut self, words: &[String], index: usize) {
		if words.len() == 1 {
			self.found_prose = true;
			self.final_prose_index_into_separated_parts = Some(self.prose_index.unwrap());
		} else {
			self.final_prose_index_into_separated_parts = Some(index + words.len() - 1);
			let separated_parts = self.text_separation.get_separated_parts();
			let final_separated_part = separated_parts[self.final_prose_index_into_separated_parts.unwrap()].to_lowercase();
			let last_word = &words[words.len() - 1];
			if final_separated_part.starts_with(last_word) {
				self.prose_ending_index = Some(last_word.len());
				self.found_prose = true;
			}
		}
	}
	fn reset_indices(&mut self) {
		self.prose_beginning_index = None;
		self.prose_index = None;
		self.prose_ending_index = None;
		self.final_prose_index_into_separated_parts = None;
	}

	fn search_for_prose_at_separated_part_index(&mut self, prose_without_spaces: &str, words: &[String], index: usize)-> () { 
		self.reset_indices();
		
		let separated_part = self.text_separation.get_separated_parts()[index].to_lowercase();

		self.search_for_prose_at_separated_part_index_beginning(prose_without_spaces, &separated_part, index);
		if self.found_prose { return (); }
		if words.len() + index > self.text_separation.get_separated_parts().len() { return (); }

		self.search_for_prose_beginning_at_separated_part_index(words, &separated_part);
		if self.prose_beginning_index.is_none() { return (); }

		if self.is_prose_middle_different_from_separated_parts_at_index(words, self.text_separation.get_separated_parts(), index) { return (); }

		self.perform_final_prose_search_at_index(words, index);
	}
}