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
use std::collections::HashSet;
use std::num::ParseIntError;
use thiserror::Error;

type Graph = DiGraph<u32, String>;

/// Deterministic Finite Automaton
struct Dfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: Graph,
    alphabet: Vec<String>,
}

/// Nondeterministic Finite Automaton
struct Nfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: Graph,
    alphabet: Vec<String>,
}

/// Deterministic Finite Automaton with Lambda transitions
struct LambdaDfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: Graph,
    alphabet: Vec<String>,
}

/// Nondeterministic Finite Automaton with Lambda transitions
struct LambdaNfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: Graph,
    alphabet: Vec<String>,
}

trait AutomatonFromFile: Sized {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError>;
}

impl AutomatonFromFile for Dfa {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError> {
        read_raw_data(file_path).map(Self::from)
    }
}

impl AutomatonFromFile for Nfa {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError> {
        read_raw_data(file_path).map(Self::from)
    }
}

impl AutomatonFromFile for LambdaDfa {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError> {
        read_raw_data(file_path).map(Self::from)
    }
}

impl AutomatonFromFile for LambdaNfa {
    fn try_read_from_file(file_path: impl AsRef<str>) -> Result<Self, ReadGraphError> {
        read_raw_data(file_path).map(Self::from)
    }
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

struct RawAutomaton {
    initial_state: u32,
    final_states: Vec<u32>,
    edges: Vec<(u32, u32, String)>,
    alphabet: Vec<String>,
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

impl From<RawAutomaton> for Dfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(!edge.2.is_empty(), "DFA cannot have lambda transitions");
            assert!(raw.alphabet.contains(&edge.2));
        }
        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl From<RawAutomaton> for Nfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(!edge.2.is_empty(), "NFA cannot have lambda transitions");
            assert!(raw.alphabet.contains(&edge.2));
        }
        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl From<RawAutomaton> for LambdaDfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(edge.2.is_empty() || raw.alphabet.contains(&edge.2));
        }
        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl From<RawAutomaton> for LambdaNfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(edge.2.is_empty() || raw.alphabet.contains(&edge.2));
        }
        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl Dfa {
    fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_state = NodeIndex::new(self.initial_state as usize);
        let mut current_window = input.as_ref();
        while !current_window.is_empty() {
            let mut word_len = current_window.len();
            while word_len > 0
                && !self
                    .alphabet
                    .contains(&current_window[..word_len].to_string())
            {
                word_len -= 1;
            }
            if word_len == 0 {
                return false;
            }
            let (word, rest) = current_window.split_at(word_len);
            current_window = rest;
            let mut next_state = None;
            for edge in self
                .graph
                .edges_directed(current_state, Direction::Outgoing)
            {
                if *edge.weight() == word {
                    next_state = Some(edge.target());
                    break;
                }
            }
            match next_state {
                Some(state) => current_state = state,
                None => return false,
            }
        }
        self.final_states
            .contains(&u32::try_from(current_state.index()).unwrap_or(0))
    }
}

impl Nfa {
    fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(NodeIndex::new(self.initial_state as usize));
        let mut current_window = input.as_ref();
        while !current_window.is_empty() {
            let mut word_len = current_window.len();
            while word_len > 0
                && !self
                    .alphabet
                    .contains(&current_window[..word_len].to_string())
            {
                word_len -= 1;
            }
            if word_len == 0 {
                return false;
            }
            let (word, rest) = current_window.split_at(word_len);
            current_window = rest;
            let mut next_states = HashSet::new();
            for state in &current_states {
                for edge in self.graph.edges_directed(*state, Direction::Outgoing) {
                    if *edge.weight() == word {
                        next_states.insert(edge.target());
                    }
                }
            }
            current_states = next_states;
            if current_states.is_empty() {
                return false;
            }
        }
        current_states.iter().any(|s| {
            self.final_states
                .contains(&u32::try_from(s.index()).unwrap_or(0))
        })
    }
}

impl LambdaDfa {
    fn epsilon_closure(&self, states: &HashSet<NodeIndex>) -> HashSet<NodeIndex> {
        let mut closure = states.clone();
        let mut stack: Vec<NodeIndex> = states.iter().copied().collect();
        while let Some(current) = stack.pop() {
            for edge in self.graph.edges_directed(current, Direction::Outgoing) {
                if edge.weight().is_empty() {
                    if closure.insert(edge.target()) {
                        stack.push(edge.target());
                    }
                }
            }
        }
        closure
    }

    fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(NodeIndex::new(self.initial_state as usize));
        current_states = self.epsilon_closure(&current_states);
        let mut current_window = input.as_ref();
        while !current_window.is_empty() {
            let mut word_len = current_window.len();
            while word_len > 0
                && !self
                    .alphabet
                    .contains(&current_window[..word_len].to_string())
            {
                word_len -= 1;
            }
            if word_len == 0 {
                return false;
            }
            let (word, rest) = current_window.split_at(word_len);
            current_window = rest;
            let mut next_states = HashSet::new();
            for state in &current_states {
                for edge in self.graph.edges_directed(*state, Direction::Outgoing) {
                    if *edge.weight() == word {
                        next_states.insert(edge.target());
                    }
                }
            }
            current_states = self.epsilon_closure(&next_states);
            if current_states.is_empty() {
                return false;
            }
        }
        current_states.iter().any(|s| {
            self.final_states
                .contains(&u32::try_from(s.index()).unwrap_or(0))
        })
    }
}

impl LambdaNfa {
    fn epsilon_closure(&self, states: &HashSet<NodeIndex>) -> HashSet<NodeIndex> {
        let mut closure = states.clone();
        let mut stack: Vec<NodeIndex> = states.iter().copied().collect();
        while let Some(current) = stack.pop() {
            for edge in self.graph.edges_directed(current, Direction::Outgoing) {
                if edge.weight().is_empty() {
                    if closure.insert(edge.target()) {
                        stack.push(edge.target());
                    }
                }
            }
        }
        closure
    }

    fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(NodeIndex::new(self.initial_state as usize));
        current_states = self.epsilon_closure(&current_states);
        let mut current_window = input.as_ref();
        while !current_window.is_empty() {
            let mut word_len = current_window.len();
            while word_len > 0
                && !self
                    .alphabet
                    .contains(&current_window[..word_len].to_string())
            {
                word_len -= 1;
            }
            if word_len == 0 {
                return false;
            }
            let (word, rest) = current_window.split_at(word_len);
            current_window = rest;
            let mut next_states = HashSet::new();
            for state in &current_states {
                for edge in self.graph.edges_directed(*state, Direction::Outgoing) {
                    if *edge.weight() == word {
                        next_states.insert(edge.target());
                    }
                }
            }
            current_states = self.epsilon_closure(&next_states);
            if current_states.is_empty() {
                return false;
            }
        }
        current_states.iter().any(|s| {
            self.final_states
                .contains(&u32::try_from(s.index()).unwrap_or(0))
        })
    }
}

fn main() {
    println!("--- DFA ---");
    let dfa = Dfa::try_read_from_file("dfa.txt").unwrap();
    println!("DFA 'aa': {}", dfa.run("aa"));
    println!("DFA 'ab': {}", dfa.run("ab"));

    println!("--- NFA ---");
    let nfa = Nfa::try_read_from_file("nfa.txt").unwrap();
    println!("NFA 'a': {}", nfa.run("a"));
    println!("NFA 'ab': {}", nfa.run("ab"));

    println!("--- Lambda-DFA ---");
    let lambda_dfa = LambdaDfa::try_read_from_file("lambda_dfa.txt").unwrap();
    println!("Lambda-DFA 'a': {}", lambda_dfa.run("a"));
    println!("Lambda-DFA 'ab': {}", lambda_dfa.run("ab"));

    println!("--- Lambda-NFA ---");
    let lambda_nfa = LambdaNfa::try_read_from_file("lambda_nfa.txt").unwrap();
    println!("Lambda-NFA 'a': {}", lambda_nfa.run("a"));
    println!("Lambda-NFA 'ab': {}", lambda_nfa.run("ab"));
}
