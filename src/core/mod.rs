//! Pure game engine for Acquire.
//!
//! This module contains the game logic with **zero I/O**: no printing, no
//! stdin, no sockets. The engine advances until it needs a decision, yields
//! the decision request, and resumes when a decision is applied.

pub mod bank;
pub mod board;
pub mod chains;
pub mod chains_mgr;
pub mod decider;
pub mod demo;
pub mod endcond;
pub mod fusion;
pub mod game;
pub mod log;
pub mod players;
pub mod rules;
pub mod settings;
pub mod stock;
