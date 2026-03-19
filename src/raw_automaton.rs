use itertools::Itertools;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs;
use std::io;
use thiserror::Error;

/// itermediate representation automaton (builder)
#[derive(Debug, Clone)]
pub struct RawAutomaton {
    pub initial_state: u32,
    pub final_states: Vec<u32>,
    pub edges: Vec<(u32, u32, String)>,
    pub alphabet: Vec<String>,
}

/// "state,state,symbol" -> (u32, u32, String)
#[inline]
fn convert_edge_text_line(text: impl AsRef<str>) -> Option<(u32, u32, String)> {
    let (from, to, symbol) = text.as_ref().split(',').map(str::trim).collect_tuple()?;

    let from = from.parse::<u32>().ok()?;
    let to = to.parse::<u32>().ok()?;
    let symbol = match symbol {
        "lambda" | "epsilon" => String::new(),
        _ => symbol.to_string(),
    };

    Some((from, to, symbol))
}

/// Trait for reading an automaton from a file.
pub trait Automaton: Sized + Debug + Clone {
    /// Tries to read an automaton from the given file path.
    ///
    /// # Errors
    ///
    /// Returns `ReadGraphError` if the file cannot be read or the content is invalid.
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError>;
}

impl<T: From<RawAutomaton> + Debug + Clone> Automaton for T {
    #[inline]
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

#[inline]
fn split_comma_nonempty(text: &str) -> impl Iterator<Item = String> {
    text.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_raw_data(path: impl AsRef<str>) -> Result<RawAutomaton, ReadGraphError> {
    use ReadGraphError as RGE;
    let input = fs::read_to_string(path.as_ref())?;
    let mut lines = input.lines();

    let alphabet = lines.next().ok_or(RGE::MissingAlphabet)?;
    let initial_state = lines.next().ok_or(RGE::MissingInitialState)?;
    let final_states = lines.next().ok_or(RGE::MissingFinalStates)?;
    let text_edges = lines;

    let alphabet = split_comma_nonempty(alphabet).collect();

    let initial_state = initial_state.trim().parse().map_err(|_| RGE::BadInput)?;

    let final_states = split_comma_nonempty(final_states)
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| RGE::BadInput)?;

    let edges = text_edges
        .map(convert_edge_text_line)
        .collect::<Option<_>>()
        .ok_or(RGE::BadInput)?;

    Ok(RawAutomaton {
        initial_state,
        final_states,
        edges,
        alphabet,
    })
}

pub fn advance_empty_word(
    graph: &DiGraph<u32, String>,
    states: &HashSet<NodeIndex>,
) -> HashSet<NodeIndex> {
    let mut closure = states.clone();
    let mut stack: Vec<NodeIndex> = states.iter().copied().collect();
    while let Some(current) = stack.pop() {
        for edge in graph.edges_directed(current, Direction::Outgoing) {
            if edge.weight().is_empty() && closure.insert(edge.target()) {
                stack.push(edge.target());
            }
        }
    }
    closure
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
