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
	prose: String,
}

impl TextSeparationAnalyzer {
	pub fn new_from_text(text: &str) -> Self {
		TextSeparationAnalyzer::new(text, is_character_alpha)
	}

	pub fn new(text: &str, character_filter: fn(char) -> bool) -> Self {
		TextSeparationAnalyzer {
			text_separation: TextSeparation::new(text, character_filter),
			prose_index: None,
			final_prose_index_into_separated_parts: None,
			prose_beginning_index: None,
			prose_ending_index: None,
			number_of_prose_words: None,
			found_prose: false,
			prose: String::new(),
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

	pub fn search_for_prose_in_separated_part(&mut self, prose: &str) {
		let lowercase_prose = prose.to_lowercase();
		let prose_without_spaces = lowercase_prose.replace(' ', "").to_string();
		let words: Vec<String> = lowercase_prose.split(' ').map(|s| s.to_string()).collect();
		self.number_of_prose_words = Some(words.len());
		self.prose = prose.to_string();
		for index in 0..self.text_separation.get_separated_parts().len() {
			self.search_for_prose_at_separated_part_index(&prose_without_spaces, &words, index);
			self.prose_index = Some(index);
			if self.found_prose { return (); }
		}
		self.found_prose = false;
	}

	pub fn is_separator_consistent(&self, starting_index: usize, ending_index: usize) -> bool {
		let separators = &self.text_separation.get_separators()[starting_index..ending_index];
		if separators.len() <= 1 { return true; }
		let initial_separator = &separators[0];
		for separator in &separators[1..] {
			if separator != initial_separator { return false; }
		}
		true
	}

	pub fn is_entire_text_separator_consistent(&self) -> bool {
		self.is_separator_consistent(0, self.text_separation.get_separators().len())
	}

	pub fn get_prose_index(&self) -> Option<usize> {
		self.prose_index
	}

	pub fn get_prose_beginning_index(&self) -> Option<usize> {
		self.prose_beginning_index
	}

	pub fn get_prose_ending_index(&self) -> Option<usize> {
		self.prose_ending_index
	}

	pub fn is_prose_separator_consistent(&self) -> bool {
		self.is_separator_consistent(self.prose_index.unwrap(), self.final_prose_index_into_separated_parts.unwrap())
	}

	pub fn get_first_prose_separator(&self) -> Option<&String> {
		let separators = self.text_separation.get_separators();
		if self.prose_index.unwrap() < separators.len() && self.prose_index.unwrap() != self.final_prose_index_into_separated_parts.unwrap() {
			Some(&separators[self.prose_index.unwrap()])
		} else {
			None
		}
	}

	pub fn has_found_prose(&self) -> bool {
		self.found_prose
	}

	pub fn compute_text_before_prose(&self) -> String {
		let mut text = self.text_separation.get_prefix().clone();
		let separated_parts = self.text_separation.get_separated_parts();
		let separators = self.text_separation.get_separators();
		let prose_index = self.prose_index.unwrap();
		for index in 0..prose_index {
			text.push_str(&separated_parts[index]);
			text.push_str(&separators[index]);
		}
		text.push_str(&separated_parts[prose_index][0..self.prose_beginning_index.unwrap()]);
		text
	}

	pub fn compute_text_after_prose(&self) -> String {
		let separated_parts = self.text_separation.get_separated_parts();
		let separators = self.text_separation.get_separators();
		let mut text = String::new();
		let first_word = &separated_parts[self.final_prose_index_into_separated_parts.unwrap()];
		if self.prose_ending_index.unwrap() < first_word.len() {
			text.push_str(&first_word[self.prose_ending_index.unwrap()..]);
		}
		if self.final_prose_index_into_separated_parts.unwrap() < separators.len() {
			text.push_str(&separators[self.final_prose_index_into_separated_parts.unwrap()]);
		}
		for index in (self.final_prose_index_into_separated_parts.unwrap() + 1)..separated_parts.len() {
			text.push_str(&separated_parts[index]);
			if index < separators.len() {
				text.push_str(&separators[index]);
			}
		}
		text
	}

	fn compute_prose_portion_of_nonseparated_text(&self) -> Vec<String> {
		let words: Vec<&str> = self.prose.split(' ').collect();
		let prose_portion_of_text_as_string = &self.text_separation.get_separated_parts()[self.prose_index.unwrap()][self.prose_beginning_index.unwrap()..self.prose_ending_index.unwrap()];
		let mut word_starting_index = 0;
		let mut words_from_text = Vec::new();
		for word in words {
			let word_ending_index = word_starting_index + word.len();
			let word_from_text = &prose_portion_of_text_as_string[word_starting_index..word_ending_index];
			words_from_text.push(word_from_text.to_string());
			word_starting_index = word_ending_index;
		}
		words_from_text
	}

	fn compute_prose_portion_of_separated_text(&self, prose_final_index: usize) -> Vec<String> {
		let mut prose_words = Vec::new();
		let separated_parts = self.text_separation.get_separated_parts();
		prose_words.push(separated_parts[self.prose_index.unwrap()][self.prose_beginning_index.unwrap()..].to_string());
		for index in (self.prose_index.unwrap() + 1)..prose_final_index {
			prose_words.push(separated_parts[index].clone());
		}
		prose_words.push(separated_parts[prose_final_index][..self.prose_ending_index.unwrap()].to_string());
		prose_words
	}

	pub fn compute_prose_portion_of_text(&self) -> Vec<String> {
		if self.prose_index.unwrap() == self.final_prose_index_into_separated_parts.unwrap() {
			self.compute_prose_portion_of_nonseparated_text()
		} else {
			self.compute_prose_portion_of_separated_text(self.final_prose_index_into_separated_parts.unwrap())
		}
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	fn is_consistent_separator(target_text: &str) -> bool {
		let analyzer = TextSeparationAnalyzer::new_from_text(target_text);
		analyzer.is_entire_text_separator_consistent()
	}

	#[test]
	fn test_consistent_separator_without_separators() {
		assert!(is_consistent_separator("thisisatest"));
	}

	#[test]
	fn test_handles_single_character_separator() {
		assert!(is_consistent_separator("this_is_a_test"));
	}

	#[test]
	fn test_inconsistent_separator_with_multiple_separators() {
		assert!(!is_consistent_separator("this_is__a_test"));
	}

	#[test]
	fn test_consistent_separator_with_multiple_characters() {
		assert!(is_consistent_separator("this!!!!is!!!!a!!!!test"));
	}

	fn assert_text_before_prose_matches(original_text: &str, prose: &str, expected_text_before_prose: &str) {
		let mut analyzer = TextSeparationAnalyzer::new_from_text(original_text);
		analyzer.search_for_prose_in_separated_part(prose);
		assert_eq!(analyzer.compute_text_before_prose(), expected_text_before_prose);
	}

	fn assert_text_after_prose_matches(original_text: &str, prose: &str, expected_text_after_prose: &str) {
		let mut analyzer = TextSeparationAnalyzer::new_from_text(original_text);
		analyzer.search_for_prose_in_separated_part(prose);
		assert_eq!(analyzer.compute_text_after_prose(), expected_text_after_prose);
	}

	#[test]
	fn test_can_find_empty_text_before_prose_with_one_word() {
		assert_text_before_prose_matches("test", "test", "");
	}

	#[test]
	fn test_can_find_text_before_prose_with_one_word() {
		assert_text_before_prose_matches("test", "st", "te");
	}

	#[test]
	fn test_can_find_text_before_prose_with_multiple_words() {
		assert_text_before_prose_matches(
			"_This is_a!test today",
			"a test",
			"_This is_",
		);
	}

	#[test]
	fn test_can_find_empty_text_after_prose_with_one_word() {
		assert_text_after_prose_matches(
			"test",
			"test",
			"",
		);
	}

	#[test]
	fn test_can_find_text_after_prose_with_one_word() {
		assert_text_after_prose_matches(
			"test",
			"te",
			"st",
		);
	}

	#[test]
	fn test_can_find_text_after_prose_with_multiple_words() {
		assert_text_after_prose_matches(
			"_This is_a!test today",
			"is a",
			"!test today",
		);
	}

	
}