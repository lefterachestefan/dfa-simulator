//! Automaton Simulator

#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(rustdoc::missing_crate_level_docs)]
#![forbid(clippy::all)]
#![forbid(clippy::nursery)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![forbid(clippy::style)]
#![forbid(clippy::suspicious)]
#![forbid(clippy::perf)]
#![forbid(clippy::correctness)]
#![forbid(clippy::complexity)]
#![forbid(clippy::todo)]
#![forbid(clippy::dbg_macro)]
#![forbid(clippy::missing_panics_doc)]
#![forbid(clippy::unwrap_used)]
#![forbid(clippy::absolute_paths)]
#![allow(clippy::multiple_crate_versions)]
/// Deterministic Finite Automaton module.
pub mod dfa;
/// Deterministic Finite Automaton with Lambda transitions module.
pub mod lambda_dfa;
/// Nondeterministic Finite Automaton with Lambda transitions module.
pub mod lambda_nfa;
/// Nondeterministic Finite Automaton module.
pub mod nfa;
/// Push-down Automaton module.
pub mod pda;
/// Regular Expression to Automaton conversion module.
pub mod regex;
/// Raw Automaton and parsing logic.
mod raw_automaton;

use std::{fmt::Write, fs::remove_file, io::Error, path::Path};

use petgraph::{graph::DiGraph, visit::EdgeRef};

/// General trait for all Finite Automatons.
pub trait Automaton: Sized + Clone {
    /// Runs the automaton on the given input string.
    #[must_use]
    fn run(&self, input: impl AsRef<str>) -> bool;

    /// Minimizes the automaton optimally.
    #[must_use]
    fn minimize(&self) -> Self;

    /// Saves the automaton as a PNG image using Graphviz `dot`.
    ///
    /// # Errors
    ///
    /// Returns an error if `dot` command fails or if file cannot be written.
    fn save_png(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let _ = remove_file(&path);

        let dot_content = self.to_dot();
        let mut child = Command::new("dot")
            .arg("-Tpng")
            .arg("-o")
            .arg(path.as_ref())
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::other("Failed to open stdin"))?;

        stdin.write_all(dot_content.as_bytes())?;
        drop(stdin);

        let status = child.wait()?;
        if !status.success() {
            return Err(Error::other(format!(
                "dot command failed with status {status}"
            )));
        }

        Ok(())
    }

    /// Returns the DOT representation of the automaton.
    #[must_use]
    fn to_dot(&self) -> String;
}

pub(crate) fn generate_dot(
    initial_state: u32,
    final_states: &[u32],
    graph: &DiGraph<u32, String>,
) -> String {
    let mut dot = String::from("digraph {\n    rankdir=LR;\n");

    // Final states
    if !final_states.is_empty() {
        dot.push_str("    node [shape = doublecircle];");
        for state in final_states {
            write!(dot, " {state}").expect("write graph image error");
        }
        dot.push_str(";\n");
    }

    // Normal states
    dot.push_str("    node [shape = circle];\n");

    // Initial state arrow
    dot.push_str("    secret_initial_node [label=\"\", shape=none, height=0, width=0];\n");
    writeln!(dot, "    secret_initial_node -> {initial_state};").expect("write graph image error");

    // Transitions
    for edge in graph.edge_references() {
        let from = edge.source().index();
        let to = edge.target().index();
        let label = edge.weight();
        let display_label = if label.is_empty() { "ε" } else { label };
        writeln!(dot, "    {from} -> {to} [label=\"{display_label}\"];")
            .expect("write graph image error");
    }

    dot.push_str("}\n");
    dot
}

/// Imports you probably want.
pub mod prelude {
    pub use crate::Automaton;
    pub use crate::{
        dfa::Dfa,
        lambda_dfa::LambdaDfa,
        lambda_nfa::LambdaNfa,
        nfa::Nfa,
        raw_automaton::{Loadable, ReadGraphError},
    };
}
