use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, MultiIndex, IndexList, Index, IndexedMap};
use cw_controllers::{Admin, Hooks};

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
pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("hooks");


/// index defintions, the trait-impl is mostly boilerplate
pub struct GameIndexes<'a>{
    pub opponent: MultiIndex<'a, (Addr, Vec<u8>), GameState>,
}

impl<'a> IndexList<GameState> for GameIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<GameState>> + '_> {
      let v: Vec<&dyn Index<GameState>> = vec![&self.opponent];
      Box::new(v.into_iter())
    }
}

/// struct to just hold the game map
pub struct Games<'a> {
    pub states: IndexedMap<'a, (Addr, Addr), GameState, GameIndexes<'a>>,
}

impl Default for Games<'static>{
    fn default() -> Self {

        //add as many additional key indexes as desired
        let indexes = GameIndexes{
            opponent: MultiIndex::new(
                |d: &GameState, k: Vec<u8>| (d.opponent.clone(), k),
                "states",  //this needs to match the map's name
                "states__opponent",
            ),
        };
    
        Self {
            states: IndexedMap::new("states", indexes),
        }
    }
}

