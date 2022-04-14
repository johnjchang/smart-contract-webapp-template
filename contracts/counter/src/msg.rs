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
    opponent: String
  },
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