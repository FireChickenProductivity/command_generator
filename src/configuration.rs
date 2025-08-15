// Defines functions for accessing and managing configuration files.

use crate::action_records::BasicAction;
use crate::paths;
use crate::recommendation_generation::{ActionSet, compute_string_representation_of_actions};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const CONFIGURATION_DIRECTORY_NAME: &str = "configuration";
const ACTIONS_TO_REJECT_FILE_NAME: &str = "actions_to_reject.txt";
const COMMANDS_TO_REJECT_FILENAME: &str = "commands_to_reject.txt";

fn compute_configuration_directory() -> io::Result<PathBuf> {
    paths::compute_directory_under_current_directory(CONFIGURATION_DIRECTORY_NAME)
}

pub fn create_configuration_directory() -> io::Result<()> {
    let path = paths::create_directory_under_current_directory(CONFIGURATION_DIRECTORY_NAME)?;
    // create actions to reject file
    let actions_to_reject_path = path.join(ACTIONS_TO_REJECT_FILE_NAME);
    paths::create_file(&actions_to_reject_path)?;
    // create commands to reject file
    let commands_to_reject_path = path.join(COMMANDS_TO_REJECT_FILENAME);
    paths::create_file(&commands_to_reject_path)?;
    Ok(())
}

fn compute_configuration_filepath(file_name: &str) -> io::Result<PathBuf> {
    let mut file_path = compute_configuration_directory()?;
    file_path.push(file_name);
    Ok(file_path)
}

pub fn load_action_set(file_name: &str) -> ActionSet {
    let mut actions = ActionSet::new();
    let file_path = match compute_configuration_filepath(file_name) {
        Ok(path) => path,
        Err(e) => {
            println!("Error loading {}: {}", file_name, e);
            return actions;
        }
    };

    if !file_path.exists() {
        paths::warn_about_nonexistent_file(file_name);
        return actions;
    }

    if let Ok(file_content) = fs::read_to_string(file_path) {
        for line in file_content.lines() {
            actions.insert_representation(line);
        }
    } else {
        println!("Error reading {} file", file_name);
    }

    actions
}

pub fn get_actions_to_reject() -> ActionSet {
    load_action_set(ACTIONS_TO_REJECT_FILE_NAME)
}

pub fn get_commands_to_reject() -> ActionSet {
    load_action_set(COMMANDS_TO_REJECT_FILENAME)
}

pub fn append_representations(file_name: &str, representations: &Vec<String>) {
    if representations.is_empty() {
        return;
    }

    let file_path = match compute_configuration_filepath(file_name) {
        Ok(path) => path,
        Err(e) => {
            println!(
                "Error computing filepath {} for appending: {}",
                file_name, e
            );
            return;
        }
    };

    if !file_path.exists() {
        paths::warn_about_nonexistent_file(file_name);
        return;
    }

    let file_result = fs::OpenOptions::new().append(true).open(file_path);

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening file {} for appending: {}", file_name, e);
            return;
        }
    };

    for representation in representations {
        writeln!(file, "{}", representation)
            .unwrap_or_else(|e| println!("Error writing to file {}: {}", file_name, e));
    }
}

pub fn append_actions_to_reject(actions: &Vec<BasicAction>) {
    let representations: Vec<String> = actions.iter().map(|action| action.to_json()).collect();
    append_representations(ACTIONS_TO_REJECT_FILE_NAME, &representations);
}

pub fn append_commands_to_reject(commands: &Vec<Vec<BasicAction>>) {
    let representations: Vec<String> = commands
        .iter()
        .map(|command| compute_string_representation_of_actions(command))
        .collect();
    append_representations(COMMANDS_TO_REJECT_FILENAME, &representations);
}
