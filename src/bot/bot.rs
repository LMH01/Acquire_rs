//! The single-player bot.
//!
//! A [`Bot`] implements the core [`Decider`] trait, answering the engine's
//! [`InputRequest`]s with simple, explainable heuristics (see `PLAN.md` §3.4).
//! Given a seed it is deterministic, which makes headless bot-vs-bot games
//! reproducible. The bot never selects an illegal card, an unavailable chain, or a
//! purchase it cannot afford — it only ever returns options the engine presents and
//! the limits the engine exposes.

use std::cell::RefCell;
use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::core::bank::Bank;
use crate::core::board::AnalyzedPosition;
use crate::core::chains::{HotelChain, PriceLevel};
use crate::core::decider::Decider;
use crate::core::game::{Decision, Game, InputRequest};
use crate::core::rules::PlaceHotelCase;

/// A simple, seeded, deterministic bot.
pub struct Bot {
    /// The index of the player this bot controls (into [`Game::players`]).
    player_index: usize,
    /// A human-readable name for the seat.
    name: String,
    /// The seeded RNG. `RefCell` gives interior mutability so `decide(&self, ..)` can
    /// still advance the state while the [`Decider`] trait takes `&self`.
    rng: RefCell<StdRng>,
}

impl Bot {
    /// Creates a new bot for `player_index` with the given `seed`.
    pub fn new(seed: u64, player_index: usize) -> Self {
        Self {
            player_index,
            name: format!("Bot {}", player_index + 1),
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
        }
    }

    /// The default seed for a given player index (reproducible sessions).
    pub fn default_seed(player_index: usize) -> u64 {
        0x5EED + player_index as u64
    }

    /// This bot's player, from the game.
    fn me<'a>(&self, game: &'a Game) -> &'a crate::core::players::Player {
        &game.players[self.player_index]
    }

    /// Scores a (legal) card; higher is better.
    fn score_card(&self, card: &AnalyzedPosition, game: &Game) -> i64 {
        match &card.place_hotel_case {
            PlaceHotelCase::SingleHotel => 5,
            PlaceHotelCase::NewChain(positions) => {
                let mut score = 50 + 10 * positions.len() as i64;
                // A High/Medium price-level chain is still available to found.
                if let Some(available) = game.chains.available_chains() {
                    if available.iter().any(|c| c.price_level() != PriceLevel::Low) {
                        score += 50;
                    }
                }
                score
            }
            PlaceHotelCase::ExtendsChain(chain, positions) => {
                let new_len = game.chains.chain_length(chain) + positions.len() as u32;
                let price = Bank::stock_price(&game.chains, chain);
                let mut score = 100 * price as i64 + 10 * new_len as i64;
                let id = self.me(game).id;
                if game.bank.is_largest_shareholder(id, chain)
                    || game.bank.is_second_largest_shareholder(id, chain)
                {
                    score += 200;
                }
                score
            }
            PlaceHotelCase::Fusion(chains, _origin) => {
                // Deliberately low: avoid fusions unless they are large.
                let survivor = chains
                    .iter()
                    .max_by_key(|c| game.chains.chain_length(c))
                    .copied()
                    .unwrap_or_else(|| *chains.first().unwrap());
                let price = Bank::stock_price(&game.chains, &survivor);
                20 * price as i64
            }
            PlaceHotelCase::Illegal(_) => 0,
        }
    }

    /// Returns `true` if the bot holds a hand card that would extend `chain`.
    fn can_extend(&self, chain: &HotelChain, game: &Game) -> bool {
        self.me(game).analyzed_cards.iter().any(
            |c| matches!(&c.place_hotel_case, PlaceHotelCase::ExtendsChain(c2, _) if *c2 == *chain),
        )
    }
}

/// Numeric rank of a price level (higher = more valuable).
fn price_rank(level: PriceLevel) -> u8 {
    match level {
        PriceLevel::Low => 0,
        PriceLevel::Medium => 1,
        PriceLevel::High => 2,
    }
}

/// Picks the fusion survivor: the longest chain, ties broken by higher price level.
fn pick_survivor(chains: &[HotelChain], game: &Game) -> HotelChain {
    chains
        .iter()
        .max_by_key(|c| (game.chains.chain_length(c), price_rank(c.price_level())))
        .copied()
        .unwrap_or_else(|| *chains.first().unwrap())
}

impl Decider for Bot {
    fn decide(&self, request: &InputRequest, game: &Game) -> Decision {
        match request {
            InputRequest::ChooseCard { legal } => {
                let mut scored: Vec<(i64, _)> = Vec::new();
                for pos in legal {
                    if let Some(card) = self
                        .me(game)
                        .analyzed_cards
                        .iter()
                        .find(|c| &c.position == pos)
                    {
                        scored.push((self.score_card(card, game), *pos));
                    }
                }
                if scored.is_empty() {
                    // Fall back to the first offered card (should not happen for a legal list).
                    return Decision::Card(
                        *legal.first().expect("a legal card list must not be empty"),
                    );
                }
                let max = scored.iter().map(|(s, _)| *s).max().unwrap();
                let ties: Vec<_> = scored
                    .into_iter()
                    .filter(|(s, _)| *s == max)
                    .map(|(_, p)| p)
                    .collect();
                let idx = self.rng.borrow_mut().gen_range(0..ties.len());
                Decision::Card(ties[idx])
            }
            InputRequest::Pass { can_redraw: _ } => Decision::Pass { redraw: true },
            InputRequest::ChooseChain { available } => {
                let max_rank = available
                    .iter()
                    .map(|c| price_rank(c.price_level()))
                    .max()
                    .unwrap_or(0);
                let ties: Vec<_> = available
                    .iter()
                    .copied()
                    .filter(|c| price_rank(c.price_level()) == max_rank)
                    .collect();
                let idx = self.rng.borrow_mut().gen_range(0..ties.len());
                Decision::Chain(ties[idx])
            }
            InputRequest::FusionOrder { survivor, chains } => {
                if survivor.is_some() {
                    // The survivor is already fixed: order only the dead chains by ascending
                    // length (higher price level first on a tie).
                    let mut rest = chains.to_vec();
                    rest.sort_by(|a, b| {
                        game.chains
                            .chain_length(a)
                            .cmp(&game.chains.chain_length(b))
                            .then_with(|| {
                                price_rank(b.price_level()).cmp(&price_rank(a.price_level()))
                            })
                    });
                    Decision::FusionOrder(rest)
                } else {
                    // No fixed survivor: pick one (longest, tie by price level) and put it
                    // last; order the rest by ascending length.
                    let survivor = pick_survivor(chains, game);
                    let mut rest: Vec<HotelChain> =
                        chains.iter().copied().filter(|c| *c != survivor).collect();
                    rest.sort_by(|a, b| {
                        game.chains
                            .chain_length(a)
                            .cmp(&game.chains.chain_length(b))
                            .then_with(|| {
                                price_rank(b.price_level()).cmp(&price_rank(a.price_level()))
                            })
                    });
                    rest.push(survivor);
                    Decision::FusionOrder(rest)
                }
            }
            InputRequest::FusionStocks {
                dead,
                alive,
                max_exchange,
                max_sell,
                holder: _,
            } => {
                let price_dead = Bank::stock_price(&game.chains, dead);
                let price_alive = Bank::stock_price(&game.chains, alive);
                if price_alive >= 2 * price_dead && *max_exchange > 0 {
                    // Exchange as much as possible (an even number, bounded by the bank).
                    let available = *game.bank.stocks_available(alive, &game.chains);
                    let mut exchange = *max_exchange;
                    if exchange % 2 != 0 {
                        exchange -= 1;
                    }
                    exchange = exchange.min(available * 2);
                    Decision::FusionStocks { exchange, sell: 0 }
                } else {
                    Decision::FusionStocks {
                        exchange: 0,
                        sell: *max_sell,
                    }
                }
            }
            InputRequest::EndGame { .. } => Decision::EndGame(true),
            InputRequest::BuyStocks {
                table,
                remaining_slots,
            } => {
                let budget = *remaining_slots;
                let me = self.me(game);
                let id = me.id;
                let money = me.money;
                let mut result: HashMap<HotelChain, u32> = HashMap::new();
                let mut used_slots = 0u32;
                let mut spent = 0u32;

                // Limits for a purchase of `row`, accounting for the slots and money already
                // committed in this request (each row's `max_affordable` assumes the *whole*
                // bankroll, so we must track total spend ourselves).
                fn affordable(row_price: u32, remaining_money: u32) -> u32 {
                    if row_price == 0 {
                        0
                    } else {
                        remaining_money / row_price
                    }
                }

                // (a) Protect the majority bonus: 1 stock in each chain where the bot is
                //     largest shareholder and the chain has < 11 hotels.
                for row in table {
                    if used_slots >= budget {
                        break;
                    }
                    if game.bank.is_largest_shareholder(id, &row.chain)
                        && game.chains.chain_length(&row.chain) < 11
                    {
                        let take = 1u32
                            .min(budget - used_slots)
                            .min(row.available)
                            .min(row.max_affordable)
                            .min(affordable(row.price, money - spent));
                        if take > 0 {
                            result.insert(row.chain, take);
                            used_slots += take;
                            spent += take.saturating_mul(row.price);
                        }
                    }
                }
                // (b) Otherwise, grow a chain the bot can extend with a hand card.
                if used_slots == 0 {
                    for row in table {
                        if self.can_extend(&row.chain, game) {
                            let take = 2u32
                                .min(budget)
                                .min(row.available)
                                .min(row.max_affordable)
                                .min(affordable(row.price, money));
                            if take > 0 {
                                result.insert(row.chain, take);
                            }
                            break;
                        }
                    }
                }
                // (c) Otherwise pass (empty map).
                Decision::Buy(result)
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::decider::DeciderKind;
    use crate::core::game::FinalResult;
    use crate::core::settings::Settings;

    fn roster(players: usize, seed: u64) -> Vec<(String, DeciderKind)> {
        (0..players)
            .map(|i| (format!("Bot {}", i), DeciderKind::bot(seed + i as u64)))
            .collect()
    }

    fn bots(players: usize, seed: u64) -> Vec<Box<dyn Decider>> {
        (0..players)
            .map(|i| Box::new(Bot::new(seed + i as u64, i)) as Box<dyn Decider>)
            .collect()
    }

    /// Runs a complete bot-vs-bot game and returns the final result plus the finished
    /// engine (for invariant checks).
    fn run_full(seed: u64, players: usize) -> (FinalResult, Game) {
        let mut game = Game::new(Settings::default(), roster(players, seed))
            .expect("the game should construct");
        let deciders = bots(players, seed);
        let result = game
            .run_until_finished(&deciders)
            .expect("the game should finish without an engine error");
        (result, game)
    }

    /// Invariant checks from `PLAN.md` §4.4. Panics when any invariant is violated.
    fn assert_invariants(game: &Game) {
        // 1. Per-chain stock conservation: bank + all players' holdings == 25.
        for chain in HotelChain::iterator() {
            let bank = *game.bank.stocks_for_sale.stocks_for_hotel(chain);
            let held: u32 = game
                .players
                .iter()
                .map(|p| *p.owned_stocks.stocks_for_hotel(chain))
                .sum();
            assert_eq!(
                bank + held,
                25,
                "stock conservation broken for chain {}",
                chain.name()
            );
        }

        // 2. Manager/board consistency: an active chain always has at least one hotel.
        //    (A brand can span several letters and, after fusions, accumulate more than a
        //    single letter's 12 hotels, so there is no small upper bound to assert here.)
        for chain in game.chains.active_chains().iter() {
            let len = game.chains.chain_length(chain);
            assert!(
                len >= 1,
                "active chain {} must have >= 1 hotel",
                chain.name()
            );
        }

        // 3. Money bookkeeping: no overflow/wraparound (debug underflow would have panicked
        //    during the run, so a clean finish already proves no underflow).
        for p in game.players.iter() {
            assert!(
                p.money < 5_000_000,
                "player {} has an implausible money value {}",
                p.name,
                p.money
            );
        }

        // 4. Card accounting: the number of cards in play (deck + all hands + placed) can
        //    never exceed the initial 108-card deck. It can only *shrink* when a player
        //    passes with a redraw (their hand is discarded, per the original rules), so
        //    this bounds against card-duplication bugs.
        let hands: u32 = game
            .players
            .iter()
            .map(|p| p.analyzed_cards.len() as u32)
            .sum();
        let total = game.deck.len() as u32 + hands + game.board.placed_count();
        assert!(
            total <= 108,
            "card accounting: deck({}) + hands({}) + placed({}) = {} exceeds the 108-card deck",
            game.deck.len(),
            hands,
            game.board.placed_count(),
            total
        );

        // 5. Turn order: the current seat is always a valid index (kept in range by the
        //    modulo advance in the engine).
        assert!(
            game.current_player < game.players.len(),
            "current_player {} out of range for {} players",
            game.current_player,
            game.players.len()
        );
    }

    #[test]
    fn bot_never_illegal() {
        let (result, game) = run_full(0x5EED, 4);
        assert_invariants(&game);
        assert!(
            !result.ranking.is_empty(),
            "a finished game must produce a ranking"
        );
        assert_eq!(
            result.ranking[0].0, result.winner,
            "the top of the ranking should be the declared winner"
        );
    }

    #[test]
    fn full_game_2p_4p_6p() {
        for players in [2usize, 4, 6] {
            let (_, game) = run_full(0xABCD, players);
            assert_invariants(&game);
        }
    }

    #[test]
    fn fusion_exercises() {
        // Run several seeded games; any fusions that occur must be handled legally and the
        // resulting state must satisfy all invariants.
        for seed in [1u64, 2, 3, 4, 5, 7, 11] {
            let (_, game) = run_full(seed, 4);
            assert_invariants(&game);
        }
    }

    #[test]
    fn buy_stock_limits() {
        // A purchase that exceeded `available`/`max_affordable`/slots would make the engine
        // return `Err` and the game would not finish. A clean finish therefore proves every
        // bot purchase stayed within the limits it was shown.
        for seed in [0x42u64, 0x99, 0x1234] {
            let (_, game) = run_full(seed, 5);
            assert_invariants(&game);
        }
    }
}
