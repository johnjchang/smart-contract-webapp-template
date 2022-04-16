#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;
use cw_controllers::Admin;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, OwnerResponse, GameMove, GameResult, AdminResponse};
use crate::state::{State, STATE, GameState, GAMES, ADMIN, HOOKS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  let state = State {
    count: msg.count,
    owner: info.sender.clone(),
  };
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  STATE.save(deps.storage, &state)?;
  ADMIN.set(deps, Some(info.sender.clone()))?;

  Ok(
    Response::new()
      .add_attribute("method", "instantiate")
      .add_attribute("owner", info.sender)
      .add_attribute("count", msg.count.to_string()),
  )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::Increment {} => try_increment(deps),
    ExecuteMsg::Decrement {} => try_decrement(deps),
    ExecuteMsg::Reset { count } => try_reset(deps, info, count),
    ExecuteMsg::UpdateOwner { address } => try_update_owner(deps, info, address),

    ExecuteMsg::StartGame { opponent, host_move } => try_start_game(deps, info, opponent, host_move),
    ExecuteMsg::UpdateAdmin { admin } => try_update_admin(deps, info, admin),
    ExecuteMsg::AddHook { hook } => {
      let hook_addr = deps.api.addr_validate(&hook)?;
      Ok(HOOKS.execute_add_hook(&ADMIN, deps, info, hook_addr)?)
    },
    ExecuteMsg::RemoveHook { hook } => {
      let hook_addr = deps.api.addr_validate(&hook)?;
      Ok(HOOKS.execute_remove_hook(&ADMIN, deps, info, hook_addr)?)
    },
    ExecuteMsg::Respond { host, opponent, opp_move } => try_respond(deps, info, host, opponent, opp_move),
  }
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
  STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    state.count += 1;
    Ok(state)
  })?;

  Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn try_decrement(deps: DepsMut) -> Result<Response, ContractError> {
  STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    state.count -= 1;
    Ok(state)
  })?;

  Ok(Response::new().add_attribute("method", "try_decrement"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
  STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    if info.sender != state.owner {
      return Err(ContractError::Unauthorized {});
    }
    state.count = count;
    Ok(state)
  })?;
  Ok(Response::new().add_attribute("method", "reset"))
}

pub fn try_update_owner(deps: DepsMut, info: MessageInfo, address: String) -> Result<Response, ContractError> {
 
  let mut state: State = STATE.load(deps.storage)?;

  // priv check
  if info.sender != state.owner {
    return Err(ContractError::Unauthorized {});
  }

  // update & persist
  state.owner = deps.api.addr_validate(&address)?;
  STATE.save(deps.storage, &state)?;
 
  Ok(Response::new().add_attribute("method", "reset"))
}

pub fn try_start_game(deps: DepsMut, info: MessageInfo, opponent: String, host_move: GameMove) -> Result<Response, ContractError> {

  //blacklist check
  if HOOKS.query_hooks(deps.as_ref())?.hooks.contains(&info.sender.to_string()){
    return Err(ContractError::Blacklist{})
  }

  let opp = deps.api.addr_validate(&opponent)?;

  let game_state: GameState = GameState{
    host: info.sender.clone(),
    opponent: opp.clone(),
    host_move: host_move.clone(),
    opp_move: None,
    result: None,
  };

  GAMES.save(deps.storage, (info.sender, opp.clone()), &game_state)?;

  Ok(Response::new().add_attributes(vec![("method", "start_game"), ("opponent", &opp.to_string()), ("host_move", &match host_move {GameMove::Rock => String::from("rock"), GameMove::Paper => String::from("paper"), GameMove::Scissors => String::from("scissors")})]))
}

pub fn try_update_admin(deps: DepsMut, info: MessageInfo, admin: String) -> Result<Response, ContractError> {
 
  //priv check
  ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

  //update admin
  let admin: Addr = deps.as_ref().api.addr_validate(&admin)?;

  ADMIN.execute_update_admin::<()>(deps, info, Some(admin))?;

  Ok(Response::new().add_attribute("method", "reset"))
}

pub fn try_respond(deps: DepsMut, info: MessageInfo, host: String, opponent: String, opp_move: GameMove) -> Result<Response, ContractError> {

  //opponent/sender check
  if info.sender != deps.api.addr_validate(&opponent)?{
    return Err(ContractError::Blacklist{})
  }

  let host_addr: Addr = deps.api.addr_validate(&host)?;
  let opp_addr: Addr = deps.api.addr_validate(&opponent)?;

  // fetch game
  let game: GameState = GAMES.load(deps.storage, (host_addr.clone(), opp_addr.clone()))?;

  let host_move: GameMove = game.host_move;

  //evaluate game result
  let mut result:GameResult = GameResult::HostWins;

  if (host_move == GameMove::Rock) && (opp_move == GameMove::Paper){
    result = GameResult::OpponentWins;
  } else if (host_move == GameMove::Paper) && (opp_move == GameMove::Scissors) {
    result = GameResult::OpponentWins;
  } else if (host_move == GameMove::Scissors) && (opp_move == GameMove::Rock) {
    result = GameResult::OpponentWins;
  } else if host_move == opp_move {
    result = GameResult::Tie;
  }

  GAMES.remove(deps.storage, (host_addr, opp_addr));

  Ok(Response::new().add_attributes(vec![("method", "start_game")]).set_data(to_binary(&result)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
  match msg {
    QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
    QueryMsg::GetAdmin {} => to_binary(&query_admin(deps)?),
  }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
  let state = STATE.load(deps.storage)?;
  Ok(CountResponse { count: state.count })
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
  let state = STATE.load(deps.storage)?;
  Ok(OwnerResponse { owner: state.owner.to_string() })
}

fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
  let admin: Option<Addr> = ADMIN.get(deps)?;

  Ok(AdminResponse { admin: match admin {
    None => String::from(""),
    _ => admin.unwrap().to_string(),

  } })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
  Ok(Response::default())
}

#[cfg(test)]
mod tests {
  use super::*;
  use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
  use cosmwasm_std::{coins, from_binary};

  #[test]
  fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(17, value.count);

    // test the owner querymsg
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
    let value: OwnerResponse = from_binary(&res).unwrap();
    assert_eq!(String::from("creator"), value.owner);

  }

  #[test]
  fn increment() {
    let mut deps = mock_dependencies(&coins(2, "token"));

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::Increment {};
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // should increase counter by 1
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(18, value.count);
  }

  #[test]
  fn decrement() {
    let mut deps = mock_dependencies(&coins(2, "token"));

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::Decrement {};
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(16, value.count);
  }

  #[test]
  fn reset() {
    let mut deps = mock_dependencies(&coins(2, "token"));

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let unauth_info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::Reset { count: 5 };
    let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    match res {
      Err(ContractError::Unauthorized {}) => {}
      _ => panic!("Must return unauthorized error"),
    }

    // only the original creator can reset the counter
    let auth_info = mock_info("creator", &coins(2, "token"));
    let msg = ExecuteMsg::Reset { count: 5 };
    let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    // should now be 5
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(5, value.count);
  }

  #[test]
  fn update_owner() {
    let mut deps = mock_dependencies(&coins(2, "token"));

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to set contract's owner to rando sender
    let unauth_info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::UpdateOwner { address: unauth_info.sender.to_string() };
    let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    match res {
      Err(ContractError::Unauthorized {}) => {}
      _ => panic!("Must return unauthorized error"),
    }

    // only the original creator can update owner 
    let auth_info = mock_info("creator", &coins(2, "token"));
    let msg = ExecuteMsg::UpdateOwner { address: String::from("anyone") };
    let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    // the owner should now be "anyone"
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
    let value: OwnerResponse = from_binary(&res).unwrap();
    assert_eq!(String::from("anyone"), value.owner);
  }

  #[test]
  fn start_game() {
    let mut deps = mock_dependencies(&coins(2, "token"));

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to set contract's owner to rando sender
    let unauth_info = mock_info("", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame { opponent: unauth_info.sender.to_string(), host_move: GameMove::Rock };
    let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);

    match res {
      Err(_err) => {}
      _ => panic!("Must return unauthorized error"),
    }

    // only a legit address can be an opponent & test rock
    let auth_info = mock_info("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame { opponent: auth_info.sender.to_string(), host_move: GameMove::Rock };
    let res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    assert_eq!(Response::new().add_attributes(vec![("method", "start_game"), ("opponent", &String::from("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9")), ("host_move", &String::from("rock"))]), res)

  }

  #[test]
  fn start_game_host_moves() {
    let mut deps = mock_dependencies(&coins(2, "token"));

    let msg = InstantiateMsg { count: 17 };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // test rock
    let auth_info = mock_info("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame { opponent: auth_info.sender.to_string(), host_move: GameMove::Rock };
    let res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    assert_eq!(Response::new().add_attributes(vec![("method", "start_game"), ("opponent", &String::from("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9")), ("host_move", &String::from("rock"))]), res);


    // test paper
    let auth_info = mock_info("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame { opponent: auth_info.sender.to_string(), host_move: GameMove::Paper };
    let res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    assert_eq!(Response::new().add_attributes(vec![("method", "start_game"), ("opponent", &String::from("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9")), ("host_move", &String::from("paper"))]), res);


    // test scissors
    let auth_info = mock_info("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame { opponent: auth_info.sender.to_string(), host_move: GameMove::Scissors };
    let res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    assert_eq!(Response::new().add_attributes(vec![("method", "start_game"), ("opponent", &String::from("terra18kgwjqrm7mcnlzcy7l8h7awnn7fs2pvdl2tpm9")), ("host_move", &String::from("scissors"))]), res);

  }
}
