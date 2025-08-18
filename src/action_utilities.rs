use crate::action_records::{Argument, BasicAction};

pub fn is_insert(action: &BasicAction) -> bool {
    if action.get_name() == "insert" && action.get_arguments().len() == 1 {
        if let Some(arg) = action.get_arguments().get(0) {
            if let Argument::StringArgument(_) = arg {
                return true;
            }
        }
    }
    false
}

pub fn create_insert_action(text: &str) -> BasicAction {
    BasicAction::new("insert", vec![Argument::StringArgument(text.to_string())])
}

/// Assumes you have made sure that the action is an insert action.
pub fn get_insert_text(action: &BasicAction) -> &String {
    if let Some(Argument::StringArgument(text)) = action.get_arguments().get(0) {
        return text;
    }
    panic!("Action is not an insert action or does not have a string argument.");
}

pub fn is_insert_only_actions(actions: &[BasicAction]) -> bool {
    actions.len() == 1 && is_insert(&actions[0])
}

/// Assumes you have made sure that the actions are insert only actions.
pub fn get_insert_text_from_insert_only_actions(actions: &[BasicAction]) -> &String {
    get_insert_text(&actions[0])
}
