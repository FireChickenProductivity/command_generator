use crate::paths;
use crate::recommendation_generation::*;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const DATA_DIRECTORY_NAME: &str = "data";

fn compute_data_directory() -> io::Result<PathBuf> {
    paths::compute_directory_under_current_directory(DATA_DIRECTORY_NAME)
}

pub fn create_data_directory() -> io::Result<()> {
    paths::create_directory_under_current_directory(DATA_DIRECTORY_NAME)?;
    Ok(())
}

pub fn output_recommendations(
    recommendations: &[CommandStatistics],
    file_name: &str,
) -> std::io::Result<()> {
    let mut file_path = compute_data_directory()?;
    file_path.push(file_name);

    let file = fs::File::create(file_path)?;
    let mut buffered_writer = io::BufWriter::new(file);

    for statistics in recommendations {
        writeln!(
            buffered_writer,
            "#Number of times used: {}",
            statistics.number_of_times_used
        )?;
        writeln!(
            buffered_writer,
            "#Number of words saved: {}",
            statistics.number_of_words_saved
        )?;
        if let Some(instantiation_set) = &statistics.instantiation_set {
            writeln!(
                buffered_writer,
                "Number of instantiations of abstract command: {}",
                instantiation_set.get_size()
            )?;
        }
        let actions = &statistics.actions;
        actions.iter().for_each(|action| {
            let action_string = action.compute_talon_script();
            writeln!(buffered_writer, "{}", action_string).unwrap();
        });
        writeln!(buffered_writer, "")?;
        writeln!(buffered_writer, "")?;
    }

    Ok(())
}
