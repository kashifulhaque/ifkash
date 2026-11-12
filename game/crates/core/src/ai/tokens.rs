//! Attack tokens (docs/game/05-enemies-and-bosses.md, "Attack tokens").
//!
//! An enemy needs a token to start an attack move against a target, and holds it until
//! the move ends (or is interrupted, or the enemy dies). At most `melee_tokens` enemies
//! hold a melee token for one target and `ranged_tokens` a ranged one, so at most that
//! many are ever in an attack move against it at once. A move that launches projectiles
//! takes from the ranged pool. Enemies whose definition sets `ignores_tokens` (bosses and
//! their adds) skip the pools.
//!
//! Tokens are counted, not stored in a pool: holders are the AI actors whose
//! `AiState::token` names the pool and whose `token_target` is the target. A holder whose move has
//! ended releases it at the start of its next AI tick, before anyone thinks, so the count
//! is never below the actors actually attacking.

use lf_content::enemy::AiTuning;

use super::{Doing, Token};
use crate::arena::Id;
use crate::world::World;

/// How many actors other than `except` hold a `pool` token against `target`.
pub fn held(world: &World, target: Id, pool: Token, except: Id) -> usize {
    world
        .actors
        .iter()
        .filter(|&(id, a)| {
            id != except
                && !a.dead
                && a.ai
                    .as_ref()
                    .is_some_and(|b| b.token == pool && b.token_target == Some(target))
        })
        .count()
}

/// The size of a pool.
pub fn limit(pool: Token, t: &AiTuning) -> usize {
    match pool {
        Token::None => usize::MAX,
        Token::Melee => usize::from(t.melee_tokens),
        Token::Ranged => usize::from(t.ranged_tokens),
    }
}

/// True if `me` could take a `pool` token against `target` now.
pub fn available(world: &World, me: Id, target: Id, pool: Token, t: &AiTuning) -> bool {
    held(world, target, pool, me) < limit(pool, t)
}

/// Ends `id`'s attack once its move is over (finished, interrupted, or dead), releasing
/// the token it held.
pub fn release_finished(world: &mut World, id: Id) {
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let current = a.moves.current.map(|c| c.num);
    let dead = a.dead;
    let Some(b) = a.ai.as_mut() else {
        return;
    };
    let attacking = match b.doing {
        Doing::Attack { move_num, .. } => !dead && current == Some(move_num),
        _ => false,
    };
    if !attacking {
        if matches!(b.doing, Doing::Attack { .. }) {
            b.doing = Doing::Nothing;
        }
        b.token = Token::None;
        b.token_target = None;
    }
}
