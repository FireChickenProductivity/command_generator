use std::collections::HashSet;

pub struct UserCommand {
    pub encountered_no: bool,
    pub encountered_yes: bool,
    pub encountered_reject_command_persistently: bool,
    pub action_number_to_reject: Option<usize>,
    pub action_number_reject_persistently: Option<usize>,
    pub encountered_accept_the_rest_of_the_commands: bool,
}
impl UserCommand {
    pub fn new(input_text: String) -> Result<UserCommand, String> {
        if input_text.is_empty() {
            return Ok(UserCommand {
                encountered_no: true,
                encountered_yes: false,
                encountered_reject_command_persistently: false,
                action_number_to_reject: None,
                action_number_reject_persistently: None,
                encountered_accept_the_rest_of_the_commands: false,
            });
        }
        let white_space_separated_tokens: Vec<&str> = input_text.split_whitespace().collect();
        if white_space_separated_tokens.len() > 2 {
            return Err(format!(
                "A valid command would only have one or fewer spaces. You entered: {}",
                input_text
            ));
        }
        let mut encountered_no = false;
        let mut encountered_yes = false;
        let mut encountered_reject_command_persistently = false;
        let mut action_number_to_reject = None;
        let mut action_number_reject_persistently = None;
        let mut expecting_action_number_to_reject = false;
        let mut expecting_action_number_reject_persistently = false;
        let mut encountered_accept_the_rest_of_the_commands = false;
        let mut invalid_character = None;
        white_space_separated_tokens[0]
            .chars()
            .for_each(|c| match c {
                'y' => {
                    encountered_yes = true;
                }
                'n' => {
                    encountered_no = true;
                }
                'r' => {
                    expecting_action_number_reject_persistently = true;
                }
                'd' => {
                    expecting_action_number_to_reject = true;
                }
                'c' => {
                    encountered_reject_command_persistently = true;
                }
                'a' => {
                    encountered_accept_the_rest_of_the_commands = true;
                }
                _ => {
                    invalid_character = Some(c);
                }
            });
        if invalid_character.is_some() {
            return Err(format!(
                "Received invalid command character: {}.",
                invalid_character.unwrap()
            ));
        }
        if encountered_no && encountered_yes {
            return Err("You cannot accept and reject a command at the same time.".to_string());
        }
        if expecting_action_number_to_reject && white_space_separated_tokens.len() < 2 {
            return Err("You need to provide an action number to reject.".to_string());
        }
        if expecting_action_number_reject_persistently && white_space_separated_tokens.len() < 2 {
            return Err("You need to provide an action number to reject persistently.".to_string());
        }
        if expecting_action_number_reject_persistently || expecting_action_number_to_reject {
            match white_space_separated_tokens[1].parse::<usize>() {
                Ok(action_number) => {
                    if expecting_action_number_reject_persistently {
                        action_number_reject_persistently = Some(action_number);
                    }
                    if expecting_action_number_to_reject {
                        action_number_to_reject = Some(action_number);
                    }
                }
                Err(_) => {
                    return Err(format!(
                        "Invalid action number: {}",
                        white_space_separated_tokens[1]
                    ));
                }
            }
        }
        return Ok(Self {
            encountered_no,
            encountered_yes,
            encountered_reject_command_persistently,
            action_number_to_reject,
            action_number_reject_persistently,
            encountered_accept_the_rest_of_the_commands,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compute_persistent_rejection_string(action_number: usize) -> String {
        format!("reject_action_persistently_{}", action_number)
    }
    fn compute_rejection_string(action_number: usize) -> String {
        format!("reject_action_{}", action_number)
    }
    fn compute_flags(command: &UserCommand) -> HashSet<String> {
        let mut flags = HashSet::new();
        if command.encountered_no {
            flags.insert("no".to_string());
        }
        if command.encountered_yes {
            flags.insert("yes".to_string());
        }
        if command.encountered_reject_command_persistently {
            flags.insert("reject_command_persistently".to_string());
        }
        if let Some(action_number) = command.action_number_to_reject {
            flags.insert(compute_rejection_string(action_number));
        }
        if let Some(action_number) = command.action_number_reject_persistently {
            flags.insert(compute_persistent_rejection_string(action_number));
        }
        if command.encountered_accept_the_rest_of_the_commands {
            flags.insert("accept_the_rest_of_the_commands".to_string());
        }
        flags
    }

    fn assert_input_has_flags(input: &str, expected_flags: &HashSet<String>) {
        let command = UserCommand::new(input.to_string()).unwrap();
        let flags = compute_flags(&command);
        assert_eq!(flags, *expected_flags);
    }

    fn assert_error(input: &str) {
        let result = UserCommand::new(input.to_string());
        assert!(result.is_err(), "Expected an error for input: {}", input);
    }

    #[test]
    fn handles_empty() {
        let input = "";
        let expected_flags = HashSet::from(["no".to_string()]);
        assert_input_has_flags(&input, &expected_flags);
    }

    #[test]
    fn rejects_two_spaces() {
        let input = "yr 3 9";
        assert_error(input);
    }

    #[test]
    fn handles_yes() {
        let input = "y";
        let expected_flags = HashSet::from(["yes".to_string()]);
        assert_input_has_flags(&input, &expected_flags);
    }

    #[test]
    fn rejects_yes_and_no() {
        let input = "yn";
        assert_error(input);
    }

    #[test]
    fn handles_no() {
        let input = "n";
        let expected_flags = HashSet::from(["no".to_string()]);
        assert_input_has_flags(&input, &expected_flags);
    }

    #[test]
    fn handles_persistently_reject_number_one() {
        let input = "r 1";
        let expected_flags = HashSet::from([compute_persistent_rejection_string(1).to_string()]);
        assert_input_has_flags(&input, &expected_flags);
    }

    #[test]
    fn handles_reject_number_two() {
        let input = "d 2";
        let expected_flags = HashSet::from([compute_rejection_string(2).to_string()]);
        assert_input_has_flags(&input, &expected_flags);
    }
}
