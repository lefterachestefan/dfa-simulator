//! Automaton Simulator

#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(clippy::all)]
#![forbid(clippy::nursery)]
#![deny(clippy::pedantic)]
#![forbid(clippy::missing_panics_doc)]
#![forbid(clippy::unwrap_used)]
#![forbid(rustdoc::missing_crate_level_docs)]

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

/// Imports you probably want.
pub mod prelude {

    pub use crate::dfa::Dfa;
    pub use crate::lambda_dfa::LambdaDfa;
    pub use crate::lambda_nfa::LambdaNfa;
    pub use crate::nfa::Nfa;
    pub use crate::raw_automaton::{Automaton, ReadGraphError};
}
