use crate::raw_automaton::RawAutomaton;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

/// Deterministic Finite Automaton
pub struct Dfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: DiGraph<u32, String>,
    alphabet: Vec<String>,
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

impl Dfa {
    pub fn run(&self, input: impl AsRef<str>) -> bool {
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
