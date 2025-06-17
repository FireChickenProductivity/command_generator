use crate::recommendation_generation::*;
use std::env::current_dir;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn create_directory_if_nonexistent(directory: &PathBuf) -> io::Result<()> {
    if !directory.exists() {
        fs::create_dir_all(&directory)?;
    }
    Ok(())
}

fn compute_data_directory() -> io::Result<PathBuf> {
    let mut data_directory = current_dir()?;
    data_directory.push("data");
    Ok(data_directory)
}

pub fn create_data_directory() -> io::Result<()> {
    let path = compute_data_directory()?;
    create_directory_if_nonexistent(&path)?;
    Ok(())
}

pub fn output_recommendations(
    recommendations: &[Information],
    file_name: &str,
) -> std::io::Result<()> {
    let mut file_path = compute_data_directory()?;
    file_path.push(file_name);

    let file = fs::File::create(file_path)?;
    let mut buffered_writer = io::BufWriter::new(file);

    for recommendation in recommendations {
        let statistics = get_information_statistics(recommendation);

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
        if let Information::Abstract(info) = recommendation {
            writeln!(
                buffered_writer,
                "Number of instantiations of abstract command: {}",
                info.get_number_of_instantiations()
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
