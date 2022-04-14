use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::msg::{GameResult, GameMove};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// count state
    pub count: i32,

    /// contract owner
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {

    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove, 
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

pub const STATE: Item<State> = Item::new("state");
pub const GAMES: Map<Addr, GameState> = Map::new("games");