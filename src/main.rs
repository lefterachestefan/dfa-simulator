//! Deterministic Finite Automaton Simulator

#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(clippy::all)]
#![forbid(clippy::nursery)]
#![deny(clippy::pedantic)]
#![forbid(clippy::missing_panics_doc)]
#![forbid(clippy::unwrap_used)]

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::num::ParseIntError;
use thiserror::Error;

struct Dfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: DiGraph<u32, String>,
    alphabet: Vec<String>,
}

fn convert_edge_text_line(text: impl AsRef<str>) -> Option<(u32, u32, String)> {
    let mut splitted = text.as_ref().split(',');
    let from = splitted.next()?;
    let to = splitted.next()?;
    let symbol = splitted.next()?;

    let from = from.trim().parse::<u32>().ok()?;
    let to = to.trim().parse::<u32>().ok()?;
    let symbol = symbol.to_string();

    Some((from, to, symbol))
}

#[derive(Error, Debug)]
enum ReadGraphError {
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

fn read_graph(path: impl AsRef<str>) -> Result<Dfa, ReadGraphError> {
    let input = std::fs::read_to_string(path.as_ref())?;
    let mut lines = input.lines();

    let alphabet = lines.next().ok_or(ReadGraphError::MissingAlphabet)?;
    let initial_state = lines.next().ok_or(ReadGraphError::MissingInitialState)?;
    let final_states = lines.next().ok_or(ReadGraphError::MissingFinalStates)?;
    let text_edges = lines;

    let alphabet: Vec<String> = alphabet
        .split(',')
        .map(std::string::ToString::to_string)
        .collect();
    let initial_state = initial_state
        .trim()
        .parse::<u32>()
        .map_err(|_| ReadGraphError::BadInput)?;
    let final_states: Vec<u32> = final_states
        .split(',')
        .map(|x| x.trim().parse::<u32>())
        .collect::<Result<Vec<u32>, ParseIntError>>()
        .map_err(|_| ReadGraphError::BadInput)?;
    let edges = text_edges
        .map(convert_edge_text_line)
        .collect::<Option<Vec<(u32, u32, String)>>>()
        .ok_or(ReadGraphError::BadInput)?;
    Ok(Dfa::new(initial_state, edges, final_states, alphabet))
}

impl Dfa {
    /// Create a new Deterministic Finite Automaton.
    /// `initial_state` - starting position, usually 0 or 1
    /// `edges` - list of possible transitions of the form: (from, to, symbol)
    /// `final_states` - accepted states
    /// `alphabet` - used symbols
    fn new(
        initial_state: u32,
        edges: Vec<(u32, u32, String)>,
        final_states: Vec<u32>,
        alphabet: Vec<String>,
    ) -> Self {
        for edge in &edges {
            assert!(alphabet.contains(&edge.2));
        }
        let graph = DiGraph::<u32, String>::from_edges(edges);

        Self {
            initial_state,
            final_states,
            graph,
            alphabet,
        }
    }

    /// Simulate the DFA on the word `input`.
    fn run(&self, input: &str) -> bool {
        println!("rulăm pe input: {input}");
        let mut current_state = NodeIndex::new(self.initial_state as usize);
        let mut current_window = input;

        while !current_window.is_empty() {
            // NOTE: this could've been a binary search but using linear for simplicity
            let mut word_len = current_window.len();
            while word_len > 0
                && !self
                    .alphabet
                    .contains(&current_window[..word_len].to_string())
            {
                word_len -= 1;
            }
            let (word, rest) = current_window.split_at(word_len);
            current_window = rest;
            let word = word.to_string();

            let mut next_state = None;
            println!("suntem în q{:?}", current_state.index());
            for edge in self
                .graph
                .edges_directed(current_state, Direction::Outgoing)
            {
                if *edge.weight() == word {
                    let target = edge.target();
                    println!("am citit {word}");
                    println!("mergem în q{:?}", target.index());
                    next_state = Some(target);
                    break;
                }
            }

            println!();
            match next_state {
                Some(state) => current_state = state,
                None => return false, // NOTE: assuming missing edge means word rejection
            }
        }

        println!("------------");
        let index = u32::try_from(current_state.index()).expect("overflow");
        self.final_states.contains(&index)
    }
}

fn main() {
    const FILE_NAME: &str = "input.txt";
    let dfa = read_graph(FILE_NAME).expect("failed to create graph from file");

    println!("{}", dfa.run("waterwater"));
    println!("{}", dfa.run("water22water22water22"));
    println!(
        "{}",
        dfa.run("waterwaterwaterwaterwater22water22water22water22water22water22")
    );
    println!("{}", dfa.run("waterwater22"));
    println!("{}", dfa.run("waterwaterwater22"));
}
