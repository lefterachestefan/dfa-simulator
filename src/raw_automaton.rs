use std::num::ParseIntError;
use thiserror::Error;

pub struct RawAutomaton {
    pub initial_state: u32,
    pub final_states: Vec<u32>,
    pub edges: Vec<(u32, u32, String)>,
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

pub trait AutomatonFromFile: Sized {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError>;
}

impl<T: From<RawAutomaton>> AutomatonFromFile for T {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError> {
        read_raw_data(file_path).map(Self::from)
    }
}

#[derive(Error, Debug)]
pub enum ReadGraphError {
    #[error("read file error")]
    FileReadFailure(#[from] std::io::Error),
    #[error("bad input")]
    BadInput,
    #[error("missing alphabet")]
    MissingAlphabet,
    #[error("missing initial state")]
    MissingInitialState,
    #[error("missing final states")]
    MissingFinalStates,
}

fn read_raw_data(path: impl AsRef<str>) -> Result<RawAutomaton, ReadGraphError> {
    let input = std::fs::read_to_string(path.as_ref())?;
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
