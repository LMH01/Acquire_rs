//! Pure fusion-order resolution.
//!
//! The original `resolve_fusion_order` / `resolve_fusion_order_three_and_four_chains`
//! interleaved prompting the player with computing the order. Here we only compute
//! *what* decision is needed; the [`super::game::Game`] state machine emits the
//! matching [`super::game::InputRequest`] and records the player's answer.

use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;
use super::rules::longest_chain;

/// The result of trying to determine the fusion order without a player decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderResolution {
    /// The order is fully determined by the chain lengths.
    /// `order` lists the chains that are fused, in order (each fused into the survivor);
    /// `survivor` is the chain that survives the fusion.
    Determined { survivor: HotelChain, order: Vec<HotelChain> },
    /// A player must choose the order.
    ///
    /// `survivor` is the already-fixed survivor (the uniquely longest chain in the
    /// 3-chain case); when `None` the survivor is also chosen from `order_chains`
    /// (it will be the **last** element of the order the player returns).
    ///
    /// `order_chains` is the set of chains the player must order.
    NeedsOrder {
        survivor: Option<HotelChain>,
        order_chains: Vec<HotelChain>,
    },
}

/// Determines the fusion order for the given chains, using chain lengths where the rules
/// allow it and flagging the cases that require a player decision.
///
/// # Returns
/// * `OrderResolution::Determined` when the rules fix the order.
/// * `OrderResolution::NeedsOrder` when the fusing player must decide.
pub fn resolve_order(chains: &[HotelChain], mgr: &HotelChainManager) -> OrderResolution {
    match chains.len() {
        2 => {
            let c1 = chains[0];
            let c2 = chains[1];
            match mgr
                .chain_length(&c1)
                .cmp(&mgr.chain_length(&c2))
            {
                std::cmp::Ordering::Greater => OrderResolution::Determined {
                    survivor: c1,
                    order: vec![c2],
                },
                std::cmp::Ordering::Less => OrderResolution::Determined {
                    survivor: c2,
                    order: vec![c1],
                },
                std::cmp::Ordering::Equal => OrderResolution::NeedsOrder {
                    survivor: None,
                    order_chains: vec![c1, c2],
                },
            }
        }
        3 => {
            let c1 = chains[0];
            let c2 = chains[1];
            let c3 = chains[2];
            match longest_chain(&c1, &c2, Some(&c3), None, mgr) {
                Some(survivor) => {
                    // The uniquely longest chain survives; order the other two by length.
                    let others: Vec<HotelChain> = chains
                        .iter()
                        .copied()
                        .filter(|c| *c != *survivor)
                        .collect();
                    let a = others[0];
                    let b = others[1];
                    match mgr.chain_length(&a).cmp(&mgr.chain_length(&b)) {
                        std::cmp::Ordering::Greater => OrderResolution::Determined {
                            survivor: *survivor,
                            order: vec![b, a],
                        },
                        std::cmp::Ordering::Less => OrderResolution::Determined {
                            survivor: *survivor,
                            order: vec![a, b],
                        },
                        std::cmp::Ordering::Equal => OrderResolution::NeedsOrder {
                            survivor: Some(*survivor),
                            order_chains: vec![a, b],
                        },
                    }
                }
                // All three chains are equally long: the fusing player decides the order.
                None => OrderResolution::NeedsOrder {
                    survivor: None,
                    order_chains: vec![c1, c2, c3],
                },
            }
        }
        // 4 chains (or an unexpected number): the fusing player decides the order manually.
        _ => OrderResolution::NeedsOrder {
            survivor: None,
            order_chains: chains.to_vec(),
        },
    }
}

/// Converts a player-provided ordered list of chains into a survivor + fusion order.
///
/// `ordered` is the order the player chose; the **last** element is the survivor when
/// `fixed_survivor` is `None`, otherwise all of `ordered` are fused into `fixed_survivor`.
pub fn order_to_result(
    fixed_survivor: Option<HotelChain>,
    ordered: Vec<HotelChain>,
) -> (HotelChain, Vec<HotelChain>) {
    match fixed_survivor {
        None => {
            let survivor = *ordered.last().expect("a fusion order must not be empty");
            let order = ordered[..ordered.len() - 1].to_vec();
            (survivor, order)
        }
        Some(survivor) => (survivor, ordered),
    }
}

#[cfg(test)]
mod tests {
    use super::{order_to_result, resolve_order, OrderResolution};
    use crate::core::board::{Board, Position};
    use crate::core::chains::HotelChain;
    use crate::core::chains_mgr::HotelChainManager;

    fn mgr_with(lengths: &[(HotelChain, u32)]) -> (HotelChainManager, Board) {
        let mut mgr = HotelChainManager::new();
        let mut board = Board::new();
        let mut bank = crate::core::bank::Bank::new();
        let mut player = crate::core::players::Player::new(vec![], 0, String::from("P"));
        let mut i = 0u32;
        for (chain, len) in lengths {
            let mut positions = Vec::new();
            for _ in 0..*len {
                let letter = ('A'..='I').nth((i % 9) as usize).unwrap();
                positions.push(Position::new(letter, 1 + (i % 12)));
                i += 1;
            }
            mgr.start_chain(*chain, positions, &mut board, &mut player, &mut bank)
                .unwrap();
        }
        (mgr, board)
    }

    #[test]
    fn two_chain_determined_by_length() {
        let (mgr, _board) = mgr_with(&[
            (HotelChain::Airport, 3),
            (HotelChain::Luxor, 5),
        ]);
        assert_eq!(
            resolve_order(&[HotelChain::Airport, HotelChain::Luxor], &mgr),
            OrderResolution::Determined {
                survivor: HotelChain::Luxor,
                order: vec![HotelChain::Airport],
            }
        );
    }

    #[test]
    fn two_chain_equal_needs_order() {
        let (mgr, _board) = mgr_with(&[
            (HotelChain::Airport, 3),
            (HotelChain::Luxor, 3),
        ]);
        assert_eq!(
            resolve_order(&[HotelChain::Airport, HotelChain::Luxor], &mgr),
            OrderResolution::NeedsOrder {
                survivor: None,
                order_chains: vec![HotelChain::Airport, HotelChain::Luxor],
            }
        );
    }

    #[test]
    fn order_to_result_picks_last_as_survivor() {
        let (survivor, order) = order_to_result(
            None,
            vec![HotelChain::Airport, HotelChain::Luxor],
        );
        assert_eq!(survivor, HotelChain::Luxor);
        assert_eq!(order, vec![HotelChain::Airport]);
    }
}
