//! # Rust Library for Boolean Formulas and Entanglement
//! This library provides a set of tools for working with boolean formulas, including
//! functionalities for manipulating boolean vectors, calculating entanglement measures, and
//! evaluating boolean expressions.

pub mod auxiliar;
#[cfg(feature = "clique")]
pub(crate) mod clique_solver;
pub mod examples;
#[cfg(feature = "fmatrix")]
pub mod fmulti;
pub mod fvalue;
pub mod macros;

/// Metrics submodule containing complexity, spectral analysis, frontier and entanglement tools.
pub mod metrics;

// Re-export metric modules at the top level for backward compatibility.
pub use metrics::certificate;
#[cfg(feature = "entanglement")]
pub use metrics::entanglement;
#[cfg(feature = "entanglement")]
pub use metrics::fragmentation;
#[cfg(feature = "frontier")]
pub use metrics::frontier;
pub use metrics::influence;
pub use metrics::sensitivity;
pub use metrics::spectral;
pub use metrics::structure;

// Re-export entanglement items at the top level for convenience.
#[cfg(feature = "entanglement")]
pub use metrics::entanglement::*;
#[cfg(feature = "entanglement")]
pub use metrics::fragmentation::*;
pub use metrics::structure::*;
