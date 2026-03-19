use std::fs;
use std::io;
use std::num::ParseIntError;
use thiserror::Error;

/// itermediate representation automaton (builder)
pub struct RawAutomaton {
    /// The initial state index.
    pub initial_state: u32,
    /// List of final state indices.
    pub final_states: Vec<u32>,
    /// List of edges: (from, to, symbol).
    pub edges: Vec<(u32, u32, String)>,
    /// The alphabet of the automaton.
    pub alphabet: Vec<String>,
}

fn convert_edge_text_line(text: impl AsRef<str>) -> Option<(u32, u32, String)> {
    let mut splitted = text.as_ref().split(',');
    let from = splitted.next()?;
    let to = splitted.next()?;
    let symbol = splitted.next()?.trim();

    let from = from.trim().parse::<u32>().ok()?;
    let to = to.trim().parse::<u32>().ok()?;

    let symbol = if symbol == "lambda" || symbol == "epsilon" {
        String::new()
    } else {
        symbol.to_string()
    };

    Some((from, to, symbol))
}

/// Trait for reading an automaton from a file.
pub trait AutomatonFromFile: Sized {
    /// Tries to read an automaton from the given file path.
    ///
    /// # Errors
    ///
    /// Returns `ReadGraphError` if the file cannot be read or the content is invalid.
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError>;
}

impl<T: From<RawAutomaton>> AutomatonFromFile for T {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError> {
        read_raw_data(file_path).map(Self::from)
    }
}

/// Errors that can occur when reading an automaton from a file.
#[derive(Error, Debug)]
pub enum ReadGraphError {
    /// Failed to read the file.
    #[error("read file error")]
    FileReadFailure(#[from] io::Error),
    /// Bad input format.
    #[error("bad input")]
    BadInput,
    /// Missing alphabet definition.
    #[error("missing alphabet")]
    MissingAlphabet,
    /// Missing initial state definition.
    #[error("missing initial state")]
    MissingInitialState,
    /// Missing final states definition.
    #[error("missing final states")]
    MissingFinalStates,
}

fn read_raw_data(path: impl AsRef<str>) -> Result<RawAutomaton, ReadGraphError> {
    let input = fs::read_to_string(path.as_ref())?;
    let mut lines = input.lines();

    let alphabet = lines.next().ok_or(ReadGraphError::MissingAlphabet)?;
    let initial_state = lines.next().ok_or(ReadGraphError::MissingInitialState)?;
    let final_states = lines.next().ok_or(ReadGraphError::MissingFinalStates)?;
    let text_edges = lines;

    let alphabet: Vec<String> = alphabet
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let initial_state = initial_state
        .trim()
        .parse::<u32>()
        .map_err(|_| ReadGraphError::BadInput)?;
    let final_states: Vec<u32> = final_states
        .split(',')
        .filter(|x| !x.trim().is_empty())
        .map(|x| x.trim().parse::<u32>())
        .collect::<Result<Vec<u32>, ParseIntError>>()
        .map_err(|_| ReadGraphError::BadInput)?;
    let edges = text_edges
        .map(convert_edge_text_line)
        .collect::<Option<Vec<(u32, u32, String)>>>()
        .ok_or(ReadGraphError::BadInput)?;

    Ok(RawAutomaton {
        initial_state,
        final_states,
        edges,
        alphabet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_edge_text_line() {
        assert_eq!(
            convert_edge_text_line("0,1,a"),
            Some((0, 1, "a".to_string()))
        );
        assert_eq!(
            convert_edge_text_line(" 0 , 1 , a "),
            Some((0, 1, "a".to_string()))
        );
        assert_eq!(
            convert_edge_text_line("0,1,lambda"),
            Some((0, 1, String::new()))
        );
        assert_eq!(
            convert_edge_text_line("0,1,epsilon"),
            Some((0, 1, String::new()))
        );
        assert_eq!(convert_edge_text_line("0,1"), None);
        assert_eq!(convert_edge_text_line("a,b,c"), None);
    }

    #[test]
    fn test_read_raw_data_bad_input() {
        let result = read_raw_data("non_existent_file.txt");
        assert!(matches!(result, Err(ReadGraphError::FileReadFailure(_))));
    }
}
