const FIVE_MINUTES_IN_SECONDS: u32 = 5 * 60;
const DEFAULT_MAX_PROSE_SIZE_TO_CONSIDER: usize = 10;
use crate::action_records::{Argument, BasicAction, Command, CommandChain, Entry, TalonCapture};
use crate::action_utilities::*;
use crate::pool;
use crate::text_separation::{
    TextSeparationAnalyzer, compute_case_string_for_prose, has_valid_case,
};
use std::collections::{HashMap, HashSet};

fn compute_number_of_words(command_chain: &CommandChain) -> u32 {
    command_chain
        .get_command()
        .get_name()
        .split_whitespace()
        .count() as u32
}

#[derive(Clone, Debug)]
pub struct CommandStatistics {
    pub number_of_times_used: usize,
    pub number_of_actions: usize,
    pub total_number_of_words_dictated: u32,
}

impl CommandStatistics {
    pub fn new(number_of_actions: usize) -> Self {
        CommandStatistics {
            number_of_times_used: 0,
            number_of_actions: number_of_actions,
            total_number_of_words_dictated: 0,
        }
    }

    pub fn get_average_words_dictated(&self) -> f32 {
        if self.number_of_times_used == 0 {
            return 0.0;
        }
        self.total_number_of_words_dictated as f32 / self.number_of_times_used as f32
    }

    pub fn process_usage(&mut self, command_chain: &CommandChain) {
        self.number_of_times_used += 1;
        self.total_number_of_words_dictated += compute_number_of_words(command_chain);
    }
}

#[derive(Clone)]
pub struct PotentialCommandInformation {
    actions: Vec<BasicAction>,
    statistics: CommandStatistics,
    chain: Option<usize>,
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
        let number_of_actions = compute_number_of_actions(&actions);
        PotentialCommandInformation {
            actions,
            statistics: CommandStatistics::new(number_of_actions),
            chain: None,
        }
    }

    pub fn get_statistics(&self) -> &CommandStatistics {
        &self.statistics
    }

    pub fn get_actions(&self) -> &Vec<BasicAction> {
        &self.actions
    }

    pub fn process_usage(&mut self, command_chain: &CommandChain) {
        if self.should_process_usage(command_chain.get_chain_number()) {
            self.process_relevant_usage(command_chain);
        }
    }

    fn should_process_usage(&self, chain: usize) -> bool {
        match self.chain {
            Some(existing_chain) => existing_chain < chain,
            None => true,
        }
    }

    fn process_relevant_usage(&mut self, command_chain: &CommandChain) {
        self.chain = Some(command_chain.get_chain_ending_index());
        self.statistics.process_usage(command_chain);
    }

    pub fn get_number_of_words_saved(&self) -> u32 {
        self.statistics.number_of_times_used as u32
            * (self.statistics.get_average_words_dictated() as u32 - 1)
    }
}

#[derive(Clone)]
pub struct ActionSet {
    set: HashSet<String>,
}

pub fn compute_string_representation_of_actions(actions: &Vec<BasicAction>) -> String {
    actions
        .iter()
        .map(|action| action.to_json())
        .collect::<Vec<String>>()
        .join("")
}

pub fn compute_string_representation_of_chain_actions(command_chain: &CommandChain) -> String {
    let actions = command_chain.get_command().get_actions();
    compute_string_representation_of_actions(actions)
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

#[derive(Clone)]
pub struct PotentialAbstractCommandInformation {
    instantiation_set: ActionSet,
    number_of_words_saved: u32,
    info: PotentialCommandInformation,
}

impl PotentialAbstractCommandInformation {
    pub fn new(instantiation: AbstractCommandInstantiation) -> Self {
        let actions = instantiation.command_chain.get_command().get_actions();
        let potential_command_information = PotentialCommandInformation::new(actions.clone());
        let mut result = Self {
            instantiation_set: ActionSet::new(),
            number_of_words_saved: instantiation.words_saved,
            info: potential_command_information,
        };
        result.process_usage(instantiation);
        result
    }

    pub fn process_usage(&mut self, instantiation: AbstractCommandInstantiation) {
        let chain_number = instantiation.command_chain.get_chain_number();
        if self.info.should_process_usage(chain_number) {
            let actions = instantiation.concrete_command.get_command().get_actions();
            self.instantiation_set.insert(&actions);
            self.info
                .process_relevant_usage(&instantiation.command_chain);
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

#[derive(Clone)]
pub enum Information {
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
    let new_command = Command::new(
        new_name,
        new_actions,
        command_chain.get_command().get_seconds_since_last_action(),
    );
    CommandChain::new(
        new_command,
        command_chain.get_chain_number(),
        command_chain.get_size(),
    )
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

    let new_command = Command::new(
        command_chain.get_command().get_name(),
        new_actions,
        command_chain.get_command().get_seconds_since_last_action(),
    );
    CommandChain::new(
        new_command,
        command_chain.get_chain_number(),
        command_chain.get_size(),
    )
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

    let new_command = Command::new(
        command_chain.get_command().get_name(),
        new_actions,
        command_chain.get_command().get_seconds_since_last_action(),
    );
    CommandChain::new(
        new_command,
        command_chain.get_chain_number(),
        command_chain.get_size(),
    )
}

fn should_make_abstract_repeat_representation(command_chain: &CommandChain) -> bool {
    let actions = command_chain.get_command().get_actions();
    if actions.len() <= 2 {
        return false;
    }
    actions.iter().any(|action| action.get_name() == "repeat")
}

fn make_abstract_repeat_representation_for(
    command_chain: &CommandChain,
) -> AbstractCommandInstantiation {
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

    let new_command =
        compute_command_chain_copy_with_new_name_and_actions(command_chain, &new_name, new_actions);
    let words_saved = compute_number_of_words(command_chain) - 2;
    AbstractCommandInstantiation {
        command_chain: new_command,
        concrete_command: command_chain.clone(),
        words_saved: words_saved as u32,
    }
}

struct ProseMatch {
    analyzer: TextSeparationAnalyzer,
    name: String,
}

fn make_abstract_representation_for_prose_command(
    command_chain: &CommandChain,
    prose_match: &ProseMatch,
    insert_to_modify_index: usize,
) -> AbstractCommandInstantiation {
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
            Argument::CaptureArgument(prose_argument),
            Argument::StringArgument(compute_case_string_for_prose(&analyzer)),
            Argument::StringArgument(analyzer.get_first_prose_separator()),
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

    let new_command = compute_command_chain_copy_with_new_name_and_actions(
        command_chain,
        &prose_match.name,
        new_actions,
    );
    let words_saved = compute_number_of_words(&new_command) - 2;
    AbstractCommandInstantiation {
        command_chain: new_command,
        concrete_command: command_chain.clone(),
        words_saved: words_saved as u32,
    }
}

struct InsertAction<'a> {
    text: &'a String,
    index: usize,
}

fn obtain_inserts_from_command_chain(command_chain: &CommandChain) -> Vec<InsertAction> {
    command_chain
        .get_command()
        .get_actions()
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            if is_insert(action) {
                let insert_text = get_insert_text(action);
                Some(InsertAction {
                    text: insert_text,
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

fn compute_text_analyzer_for_prose_and_insert(
    prose: &str,
    insert: &InsertAction,
) -> TextSeparationAnalyzer {
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
    //Can stop early if any word from prose is missing from the insert text
    let insert_text = insert.text.to_lowercase();
    for word in words[starting_index..starting_index + prose_size].iter() {
        if !insert_text.contains(&word.to_lowercase()) {
            return Err(());
        }
    }

    let prose = generate_prose_from_words(words, starting_index, prose_size);
    let analyzer = compute_text_analyzer_for_prose_and_insert(&prose, insert);
    if analyzer.has_found_prose()
        && analyzer.is_prose_separator_consistent()
        && has_valid_case(&analyzer)
    {
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
        if let Ok(match_found) = find_prose_match_for_command_given_insert_at_interval(
            words,
            insert,
            starting_index,
            prose_size,
        ) {
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
        matches.extend(
            find_prose_matches_for_command_given_insert_at_starting_index(
                &words,
                insert,
                starting_index,
                max_prose_size_to_consider,
            ),
        );
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
    let prose_matches = find_prose_matches_for_command_given_insert(
        command_chain,
        insert,
        max_prose_size_to_consider,
    );
    for match_found in prose_matches {
        let abstract_representation = make_abstract_representation_for_prose_command(
            &command_chain,
            &match_found,
            insert.index,
        );
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
        let representations_given_insert =
            make_abstract_prose_representations_for_command_given_insert(
                command_chain,
                insert,
                max_prose_size_to_consider,
            );
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
        make_abstract_prose_representations_for_command_given_inserts(
            command_chain,
            &inserts,
            max_prose_size_to_consider,
        )
    }
}

fn basic_concrete_command_filter(info: &CommandStatistics, number_of_words_saved: u32) -> bool {
    number_of_words_saved > 0
        && info.number_of_times_used > 1
        && (info.number_of_actions as f32 / info.get_average_words_dictated() < 2.0
            || info.number_of_actions as f32 * (info.number_of_times_used as f32).sqrt()
                > info.get_average_words_dictated())
}

fn basic_abstract_command_filter(info: &PotentialAbstractCommandInformation) -> bool {
    if info
        .get_potential_command_information()
        .get_statistics()
        .get_average_words_dictated()
        < 2.0
        || info.get_number_of_instantiations() <= 2
        || info.get_number_of_words_saved() < 1
    {
        return false;
    }
    basic_concrete_command_filter(
        info.get_potential_command_information().get_statistics(),
        info.get_number_of_words_saved(),
    )
}

fn is_command_after_chain_start_exceeding_time_gap_threshold(
    record_entry: &Command,
    chain_start_index: usize,
    current_chain_index: usize,
) -> bool {
    match record_entry.get_seconds_since_last_action() {
        Some(seconds) => {
            current_chain_index > chain_start_index && seconds > FIVE_MINUTES_IN_SECONDS
        }
        None => false,
    }
}

fn should_command_chain_not_cross_entry_at_record_index(
    record: &[Entry],
    chain_start_index: usize,
    current_chain_index: usize,
) -> bool {
    let record_entry = &record[current_chain_index];
    match record_entry {
        Entry::RecordingStart => true,
        Entry::Command(record_entry) => is_command_after_chain_start_exceeding_time_gap_threshold(
            &record_entry,
            chain_start_index,
            current_chain_index,
        ),
    }
}

fn add_next_record_command_to_chain(record: &[Entry], command_chain: &mut CommandChain) {
    match &record[command_chain.get_next_chain_index()] {
        Entry::Command(command) => {
            command_chain.append_command(command.clone());
        }
        _ => {
            panic!(
                "Expected a command entry at index {}, but found something else.",
                command_chain.get_next_chain_index()
            );
        }
    }
}

fn simplify_command_chain(command_chain: &CommandChain) -> CommandChain {
    let mut simplified_chain = compute_insert_simplified_command_chain(command_chain);
    simplified_chain = compute_repeat_simplified_command_chain(&simplified_chain);
    simplified_chain
}

pub fn create_abstract_commands(command_chain: &CommandChain) -> Vec<AbstractCommandInstantiation> {
    let mut commands = make_abstract_prose_representations_for_command(
        command_chain,
        DEFAULT_MAX_PROSE_SIZE_TO_CONSIDER,
    );
    if should_make_abstract_repeat_representation(command_chain) {
        let abstract_repeat_representation = make_abstract_repeat_representation_for(command_chain);
        commands.push(abstract_repeat_representation);
    }
    commands
}

fn process_abstract_command_usage(
    abstract_commands: &mut HashMap<String, PotentialAbstractCommandInformation>,
    instantiation: AbstractCommandInstantiation,
) {
    let representation =
        compute_string_representation_of_chain_actions(&instantiation.command_chain);
    if let Some(info) = abstract_commands.get_mut(&representation) {
        info.process_usage(instantiation);
    } else {
        abstract_commands.insert(
            representation,
            PotentialAbstractCommandInformation::new(instantiation),
        );
    }
}

pub fn handle_needed_abstract_commands(
    abstract_commands: &mut HashMap<String, PotentialAbstractCommandInformation>,
    command_chain: &CommandChain,
) {
    let abstractions = create_abstract_commands(command_chain);
    for abstract_command in abstractions {
        process_abstract_command_usage(abstract_commands, abstract_command);
    }
}

fn process_concrete_command_usage(
    concrete_commands: &mut HashMap<String, PotentialCommandInformation>,
    command_chain: &CommandChain,
) {
    let representation = compute_string_representation_of_chain_actions(command_chain);
    if let Some(info) = concrete_commands.get_mut(&representation) {
        info.process_usage(command_chain);
    } else {
        let mut concrete_info =
            PotentialCommandInformation::new(command_chain.get_command().get_actions().clone());
        concrete_info.process_relevant_usage(command_chain);
        concrete_commands.insert(representation, concrete_info);
    }
}

fn process_insert_action(
    simplified_command_chain: &CommandChain,
    insert: &InsertAction,
    abstract_commands: &mut HashMap<String, PotentialAbstractCommandInformation>,
) {
    let dictation = simplified_command_chain.get_command().get_name();
    let words: Vec<&str> = dictation.split_whitespace().collect();
    for starting_index in 0..words.len() {
        let maximum_size = DEFAULT_MAX_PROSE_SIZE_TO_CONSIDER.min(words.len() - starting_index + 1);
        for prose_size in 1..maximum_size {
            if let Ok(match_found) = find_prose_match_for_command_given_insert_at_interval(
                &words,
                &insert,
                starting_index,
                prose_size,
            ) {
                let abstract_representation = make_abstract_representation_for_prose_command(
                    &simplified_command_chain,
                    &match_found,
                    insert.index,
                );
                if is_acceptable_abstract_representation(&abstract_representation.command_chain) {
                    process_abstract_command_usage(abstract_commands, abstract_representation);
                }
            } else {
                break;
            }
        }
    }
}

fn create_insert_action_iterator(
    command_chain: &CommandChain,
) -> impl Iterator<Item = InsertAction> {
    command_chain
        .get_command()
        .get_actions()
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            if is_insert(action) {
                let insert_text = get_insert_text(action);
                let insert = InsertAction {
                    text: insert_text,
                    index,
                };
                Some(insert)
            } else {
                None
            }
        })
}

fn create_commands(record: &[Entry], max_chain_size: u32) -> GeneratedCommands {
    let mut concrete_commands = HashMap::new();
    let mut abstract_commands = HashMap::new();
    // let pool = pool::ThreadPool::create_with_max_threads();
    for chain in 0..record.len() {
        println!("Processing chain {}/{}", chain + 1, record.len());
        let target = record.len().min(chain + max_chain_size as usize);
        let mut command_chain = CommandChain::empty(chain);
        for chain_ending_index in chain..target {
            if should_command_chain_not_cross_entry_at_record_index(
                record,
                chain,
                chain_ending_index,
            ) {
                break;
            }
            add_next_record_command_to_chain(record, &mut command_chain);
            let simplified_command_chain = simplify_command_chain(&command_chain);
            process_concrete_command_usage(&mut concrete_commands, &simplified_command_chain);
            let insert_iterator = create_insert_action_iterator(&simplified_command_chain);
            insert_iterator.for_each(|insert| {
                process_insert_action(&simplified_command_chain, &insert, &mut abstract_commands);
            });
            if should_make_abstract_repeat_representation(&simplified_command_chain) {
                let abstract_repeat_representation =
                    make_abstract_repeat_representation_for(&simplified_command_chain);
                process_abstract_command_usage(
                    &mut abstract_commands,
                    abstract_repeat_representation,
                );
            }
        }
    }
    GeneratedCommands {
        concrete: concrete_commands
            .values()
            .filter(|info| {
                basic_concrete_command_filter(
                    info.get_statistics(),
                    info.get_number_of_words_saved(),
                )
            })
            .cloned()
            .collect(),
        abs: abstract_commands
            .values()
            .filter(|info| basic_abstract_command_filter(info))
            .cloned()
            .collect(),
    }
}

pub fn compare_information(a: &Information, b: &Information) -> std::cmp::Ordering {
    let a_info = match a {
        Information::Concrete(info) => info.get_statistics(),
        Information::Abstract(info) => &info.get_potential_command_information().get_statistics(),
    };
    let b_info = match b {
        Information::Concrete(info) => info.get_statistics(),
        Information::Abstract(info) => &info.get_potential_command_information().get_statistics(),
    };
    b_info
        .number_of_times_used
        .cmp(&a_info.number_of_times_used)
}

pub struct GeneratedCommands {
    pub concrete: Vec<PotentialCommandInformation>,
    pub abs: Vec<PotentialAbstractCommandInformation>,
}

pub fn create_sorted_info(commands: &GeneratedCommands) -> Vec<Information> {
    let mut sorted_info = Vec::new();
    sorted_info.extend(
        commands
            .concrete
            .iter()
            .map(|info| Information::Concrete(info.clone())),
    );
    sorted_info.extend(
        commands
            .abs
            .iter()
            .map(|info| Information::Abstract(info.clone())),
    );
    sorted_info.sort_by(compare_information);
    sorted_info
}

pub fn compute_recommendations_from_record(
    record: &[Entry],
    max_chain_size: u32,
) -> GeneratedCommands {
    let recommendations = create_commands(record, max_chain_size);
    recommendations
}
