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
/// Raw Automaton and parsing logic.
mod raw_automaton;

/// General trait for all Finite Automatons.
pub trait Automaton: Sized + Clone {
    /// Runs the automaton on the given input string.
    #[must_use]
    fn run(&self, input: impl AsRef<str>) -> bool;

    /// Minimizes the automaton optimally.
    #[must_use]
    fn minimize(&self) -> Self;
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
