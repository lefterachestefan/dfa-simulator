use crate::raw_automaton::RawAutomaton;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

/// Deterministic Finite Automaton
#[derive(Debug, Clone)]
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
    /// Runs the DFA on the given input string.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfa_accepts() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![1],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![(0, 1, "a".to_string()), (1, 0, "b".to_string())],
        };
        let dfa = Dfa::from(raw);
        assert!(dfa.run("a"));
        assert!(dfa.run("aba"));
        assert!(dfa.run("ababa"));
    }

    #[test]
    fn test_dfa_rejects() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![1],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![(0, 1, "a".to_string()), (1, 0, "b".to_string())],
        };
        let dfa = Dfa::from(raw);
        assert!(!dfa.run(""));
        assert!(!dfa.run("b"));
        assert!(!dfa.run("ab"));
        assert!(!dfa.run("aa"));
    }

    #[test]
    fn test_dfa_multi_char_symbol() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![1],
            alphabet: vec!["ab".to_string(), "c".to_string()],
            edges: vec![(0, 1, "ab".to_string()), (1, 0, "c".to_string())],
        };
        let dfa = Dfa::from(raw);
        assert!(dfa.run("ab"));
        assert!(dfa.run("abcab"));
        assert!(!dfa.run("a"));
        assert!(!dfa.run("b"));
        assert!(!dfa.run("abc"));
    }
}
