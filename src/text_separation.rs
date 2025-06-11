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

pub struct TextSeparationAnalyzer {
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

	pub fn get_first_prose_separator(&self) -> String {
		let separators = self.text_separation.get_separators();
		if self.prose_index.unwrap() < separators.len() && self.prose_index.unwrap() != self.final_prose_index_into_separated_parts.unwrap() {
			separators[self.prose_index.unwrap()].clone()
		} else {
			String::new()
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

fn compute_case(text: &str) -> Option<String> {
	if text.chars().all(|c| c.is_lowercase()) {
		return Some("lower".to_string());
	} else if text.chars().all(|c| c.is_uppercase()) {
		return Some("upper".to_string());
	} else if let Some(first_char) = text.chars().next() {
		if first_char.is_uppercase() && text[1..].chars().all(|c| c.is_lowercase()) {
			return Some("capitalized".to_string());
		}
	} 
	None
}

/// This is an inefficient approach that should be changed later
pub fn has_valid_case(analyzer: &TextSeparationAnalyzer) -> bool {
	analyzer.compute_prose_portion_of_text().iter().all(|word| compute_case(word).is_some())
}

fn compute_simplified_case_strings_list(case_strings: Vec<String>) -> Vec<String> {
	let mut simplified_case_strings = Vec::new();
	let mut new_case_found = false;
	let final_case = case_strings.last().unwrap().clone();
	simplified_case_strings.push(final_case);
	for case in case_strings.iter().rev().skip(1) {
		let final_case_reference = &simplified_case_strings[0];
		if case != final_case_reference || new_case_found {
			simplified_case_strings.push(case.clone());
			new_case_found = true;
		}
	}
	simplified_case_strings.reverse();
	simplified_case_strings
}

pub fn compute_case_string_for_prose(analyzer: &TextSeparationAnalyzer) -> String {
	let prose = analyzer.compute_prose_portion_of_text();
	let case_strings: Vec<String> = prose.iter()
		.filter_map(|prose_word| compute_case(prose_word))
		.collect();
	let simplified_case_strings = compute_simplified_case_strings_list(case_strings);
	let case_string = simplified_case_strings.join(" ");
	case_string
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

	fn assert_text_with_prose_gives_consistency_result(text: &str, prose: &str, expected_consistency: bool) {
		let mut analyzer = TextSeparationAnalyzer::new_from_text(text);
		analyzer.search_for_prose_in_separated_part(prose);
		assert_eq!(analyzer.is_prose_separator_consistent(), expected_consistency);
	}

	#[test]
	fn test_consistent_with_single_word_prose() {
		assert_text_with_prose_gives_consistency_result("this_is_a_test", "is", true);
	}

	#[test]
	fn test_consistent_with_snake_case_prose() {
		assert_text_with_prose_gives_consistency_result("chicken!!this_is_a_testchicken", "this is a test", true);
	}

	#[test]
	fn test_consistent_with_spaces() {
		assert_text_with_prose_gives_consistency_result("for real this is a test", "this is a test", true);
	}

	#[test]
	fn test_consistent_with_two_words() {
		assert_text_with_prose_gives_consistency_result("this_is!_@_______a_test", "is a", true);
	}

	#[test]
	fn test_consistency_handles_final_word() {
		assert_text_with_prose_gives_consistency_result("this_is_a_test!", "is a test", true);
	}

	#[test]
	fn test_inconsistent_with_two_different_separators() {
		assert_text_with_prose_gives_consistency_result("this_is a test", "this is a", false);
	}

	fn assert_prose_case_matches_expected(text: &str, prose: &str, expected_case: &str) {
		let mut analyzer = TextSeparationAnalyzer::new_from_text(text);
		analyzer.search_for_prose_in_separated_part(prose);
		let case_string = compute_case_string_for_prose(&analyzer);
		assert_eq!(case_string, expected_case);
	}

	#[test]
	fn test_case_handles_single_lower_case_word() {
		assert_prose_case_matches_expected("word", "word", "lower");
	}

	#[test]
	fn test_case_handles_single_upper_case_word() {
		assert_prose_case_matches_expected("WORD", "word", "upper");
	}

	#[test]
	fn test_case_handles_single_capitalized_word() {
		assert_prose_case_matches_expected("Word", "word", "capitalized");
	}

	#[test]
	fn test_case_handles_single_uppercase_character() {
		assert_prose_case_matches_expected("A", "a", "upper");
		
	}

	#[test]
	fn test_case_handles_camel_case() {
		assert_prose_case_matches_expected("thisIsATest", "this is a test", "lower capitalized upper capitalized");
	}

	#[test]
	fn test_case_handles_snake_case_correctly() {
		assert_prose_case_matches_expected("this_is_a_test", "this is a test", "lower");
	}

	#[test]
	fn test_case_handles_substring_prose() {
		assert_prose_case_matches_expected("yesthisIsaTESThere", "this is a test", "lower capitalized lower upper");
	}

	#[test]
	fn test_handles_separated_sub_string() {
		assert_prose_case_matches_expected("stuff!THIS_IS_A_TEST!stuff", "this is a test", "upper");
	}

	fn assert_separator_matches_expected(text: &str, prose: &str, expected_separator: &str) {
		let mut analyzer = TextSeparationAnalyzer::new_from_text(text);
		analyzer.search_for_prose_in_separated_part(prose);
		let separator = analyzer.get_first_prose_separator();
		assert_eq!(separator, expected_separator);
	}

	#[test]
	fn test_single_word_with_no_separator() {
		assert_separator_matches_expected("this", "this", "");
	}

	#[test]
	fn test_single_word_in_text_with_no_internal_separator() {
		assert_separator_matches_expected("stuff this test", "this", "");
	}

	#[test]
	fn test_handles_two_words_with_separator() {
		assert_separator_matches_expected("two  words", "two words", "  ");
	}

	#[test]
	fn test_handles_two_words_in_text_with_separator() {
		assert_separator_matches_expected("This contains two_words in the middle", "two words", "_");
	}

	#[test]
	fn test_correct_separator_with_three_words_in_text() {
		assert_separator_matches_expected("this_is_a_bigger_test_case", "a bigger test", "_");
	}

	#[test]
	fn test_handles_separator_with_two_words_at_beginning() {
		assert_separator_matches_expected("two_words_at_the_beginning", "two words", "_");
	}

	#[test]
	fn test_handles_separator_with_two_words_at_end() {
		assert_separator_matches_expected("at_ending_there_are_two_words", "two words", "_");
	}

	fn assert_indices_match(text: &str, prose: &str, prose_index: usize, prose_beginning_index: usize, prose_ending_index: usize) {
		let mut analyzer = TextSeparationAnalyzer::new_from_text(text);
		analyzer.search_for_prose_in_separated_part(prose);
		assert_eq!(analyzer.get_prose_index().unwrap(), prose_index);
		assert_eq!(analyzer.get_prose_beginning_index().unwrap(), prose_beginning_index);
		assert_eq!(analyzer.get_prose_ending_index().unwrap(), prose_ending_index);
	}

	#[test]
	fn test_can_find_one_word_prose_at_beginning() {
		assert_indices_match("testing this here", "test", 0, 0, 4);
	}

	#[test]
	fn test_can_find_multiple_words_at_beginning() {
		assert_indices_match("this_is_a_test_", "this is a test", 0, 0, 4);
	}

	#[test]
	fn test_can_find_one_word_in_middle() {
		assert_indices_match("this_is_a_realtest_right_here", "test", 3, 4, 8);
	}

	#[test]
	fn test_can_find_multiple_words_in_middle() {
		assert_indices_match("yes_forrealthis_is_a_testrighthere_", "this is a testr", 1, 7, 5);
	}

	#[test]
	fn test_can_find_one_word_at_ending() {
		assert_indices_match("this_is_actuallytestingstuff", "testing", 2, 8, 15);
	}

	#[test]
	fn test_can_find_multiple_words_at_ending() {
		assert_indices_match("once_again_this_is_a_test", "this is a test", 2, 0, 4);
	}

	#[test]
	fn can_find_multiple_words_at_middle_of_ending() {
		assert_indices_match("once_againthis_is_a_testing", "this is a test", 1, 5, 4);
	}

	
}