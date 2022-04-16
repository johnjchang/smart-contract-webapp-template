use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  /// count
  pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  /// increment count
  Increment {},

  /// decrement count
  Decrement {},

  /// reset count (contract owner only)
  Reset {
    /// count
    count: i32,
  },

  /// update owner (contract owner only)
  UpdateOwner {
    /// owner
    address: String
  },

  StartGame {
    opponent: String,
    host_move: GameMove,
  },

  UpdateAdmin{
    admin: String,
  },

  AddHook {
    hook: String,
  },

  RemoveHook{
    hook: String,
  },

  Respond{
    host: String,
    opponent: String,
    opp_move: GameMove,
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  /// @return CountResponse
  GetCount {},
  
  /// @return OwnerResponse
  GetOwner {},

  GetAdmin {},
}

/// Response type of QueryMsg.GetCount
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
  /// count property
  pub count: i32,
}

/// Response type of QueryMsg.GetOwner
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerResponse {
  /// owner property
  pub owner: String,
}

/// Response type of QueryMsg.GetAdmin
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminResponse {
  /// admin property
  pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameMove {
  Rock,
  Paper,
  Scissors,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
  HostWins,
  OpponentWins,
  Tie,
}