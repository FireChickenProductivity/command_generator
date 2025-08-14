// Defines functions for accessing and managing configuration files.

use crate::action_records::{BasicAction, load_basic_action_from_json};
use crate::paths;
use crate::recommendation_generation::ActionSet;
use std::env::current_dir;
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

pub fn get_actions_to_reject() -> ActionSet {
    let mut actions_to_reject = ActionSet::new();
    let file_path = match compute_configuration_filepath(ACTIONS_TO_REJECT_FILE_NAME) {
        Ok(path) => path,
        Err(e) => {
            println!("Error loading actions to reject: {}", e);
            return actions_to_reject;
        }
    };

    if !file_path.exists() {
        paths::warn_about_nonexistent_file(ACTIONS_TO_REJECT_FILE_NAME);
        return actions_to_reject;
    }

    let file_content = fs::read_to_string(file_path).unwrap();
    for line in file_content.lines() {
        if let Ok(action) = load_basic_action_from_json(line) {
            actions_to_reject.insert_action(&action);
        } else {
            println!("Failed to parse action from line: {}", line);
        }
    }

    actions_to_reject
}

pub fn append_actions_to_reject(actions: &Vec<BasicAction>) {
    let file_path = match compute_configuration_filepath(ACTIONS_TO_REJECT_FILE_NAME) {
        Ok(path) => path,
        Err(e) => {
            println!("Error storing actions to reject: {}", e);
            return;
        }
    };

    if !file_path.exists() {
        paths::warn_about_nonexistent_file(ACTIONS_TO_REJECT_FILE_NAME);
        return;
    }

    let file_result = fs::OpenOptions::new().append(true).open(file_path);

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening actions to reject file for appending: {}", e);
            return;
        }
    };

    for action in actions {
        writeln!(file, "{}", action.to_json())
            .unwrap_or_else(|e| println!("Error writing action to reject: {}", e));
    }
}
