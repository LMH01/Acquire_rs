//! The `Decider` abstraction: anything that can answer the engine's
//! [`InputRequest`]s.
//!
//! The engine (`core::game::Game`) never asks a human or a bot directly. It yields an
//! [`InputRequest`]; the *host* (the TUI for a human, `bot::Bot` for a bot, or a test
//! harness) turns it into a [`Decision`] and feeds it back via `Game::apply_decision`.
//!
//! This is also the clean seam for future multiplayer: a network `Decider` would
//! serialize the request over the wire and return the remote player's answer.

use crate::core::game::{Decision, Game, InputRequest};

/// Describes who sits behind a seat (used for display and logging).
///
/// The actual [`Decider`] implementation is supplied by the host; this only records the
/// *kind* of seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeciderKind {
    /// A human playing through the TUI.
    Human,
    /// A bot, seeded so its play is reproducible.
    Bot { seed: u64 },
    /// A (future) remote player over the network. Placeholder for now.
    #[allow(dead_code)]
    Network,
}

impl DeciderKind {
    /// A human seat.
    pub const HUMAN: DeciderKind = DeciderKind::Human;
    /// A bot seat with the given seed.
    pub fn bot(seed: u64) -> DeciderKind {
        DeciderKind::Bot { seed }
    }
}

/// Anything that can answer the engine's [`InputRequest`]s.
pub trait Decider: Send {
    /// Produces a [`Decision`] in response to `request`, given the current `game`.
    ///
    /// Implementations must be pure (no I/O) and must only return decisions the engine
    /// will accept.
    fn decide(&self, request: &InputRequest, game: &Game) -> Decision;

    /// A display name for the decider.
    #[allow(dead_code)]
    fn name(&self) -> &str;
}

/// A minimal, always-valid [`Decider`] used by the engine tests.
///
/// It picks the first legal option for every request and ends the game as soon as an end
/// condition is offered, which guarantees a game driven by it reaches
/// [`crate::core::game::Step::Finished`].
pub struct ScriptedDecider;

impl Decider for ScriptedDecider {
    fn decide(&self, request: &InputRequest, _game: &Game) -> Decision {
        match request {
            InputRequest::ChooseCard { legal } => Decision::Card(
                *legal
                    .first()
                    .expect("a choose-card request must list a legal card"),
            ),
            InputRequest::Pass { can_redraw: _ } => Decision::Pass { redraw: false },
            InputRequest::ChooseChain { available } => Decision::Chain(
                *available
                    .first()
                    .expect("a choose-chain request must list an available chain"),
            ),
            InputRequest::FusionOrder {
                chains,
                survivor: _,
            } => Decision::FusionOrder(chains.clone()),
            InputRequest::FusionStocks { .. } => Decision::FusionStocks {
                exchange: 0,
                sell: 0,
            },
            InputRequest::EndGame { .. } => Decision::EndGame(true),
            InputRequest::BuyStocks { .. } => Decision::Buy(std::collections::HashMap::new()),
        }
    }

    fn name(&self) -> &str {
        "scripted"
    }
}
