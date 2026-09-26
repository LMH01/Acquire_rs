//! Bot implementation for single-player mode.
//!
//! The bot answers the engine's [`crate::core::game::InputRequest`]s using simple,
//! explainable heuristics. It is deterministic given a seed.

pub mod bot;

pub use bot::Bot;
