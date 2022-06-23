#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CooldownResponse, ExecuteMsg, GridResponse, InstantiateMsg, QueryMsg};
use crate::state::{Color, Config, Dimensions, CONFIG, COOLDOWNS, DIMENSIONS, GRID};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:juno-pixel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_address = deps.api.addr_validate(&msg.admin_address)?;

    if let Some(end_height) = msg.end_height {
        if end_height <= env.block.height {
            // TODO: improve error
            return Err(ContractError::Unauthorized {});
        }
    }

    let config = Config {
        admin_address,
        cooldown: msg.cooldown,
        end_height: msg.end_height,
    };
    let dimensions = Dimensions {
        width: msg.width,
        height: msg.height,
    };
    let grid = vec![vec![Color::WHITE; msg.height as usize]; msg.width as usize];

    CONFIG.save(deps.storage, &config)?;
    DIMENSIONS.save(deps.storage, &dimensions)?;
    GRID.save(deps.storage, &grid)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Draw { x, y, color } => execute_draw(deps, env, info, x, y, color),
        _ => todo!(),
    }
}

pub fn execute_draw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    x: u64,
    y: u64,
    color: Color,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let dimensions = DIMENSIONS.load(deps.storage)?;
    let user_cooldown = COOLDOWNS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
    if x > dimensions.width || y > dimensions.height {
        // TODO: improve error
        return Err(ContractError::Unauthorized {});
    }

    if env.block.height < user_cooldown {
        // TODO: improve error
        return Err(ContractError::Unauthorized {});
    }

    if let Some(end_height) = config.end_height {
        if env.block.height > end_height {
            // TODO: improve error
            return Err(ContractError::Unauthorized {});
        }
    }

    let mut grid = GRID.load(deps.storage)?;
    grid[x as usize][y as usize] = color;

    GRID.save(deps.storage, &grid)?;
    COOLDOWNS.save(
        deps.storage,
        &info.sender,
        &(env.block.height + config.cooldown),
    )?;

    Ok(Response::new().add_attribute("action", "draw"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::GetDimensions {} => to_binary(&DIMENSIONS.load(deps.storage)?),
        QueryMsg::GetGrid {} => to_binary(&GridResponse {
            grid: GRID.load(deps.storage)?,
        }),
        QueryMsg::GetCooldown { address } => query_cooldown(deps, address),
    }
}

pub fn query_cooldown(deps: Deps, address: String) -> StdResult<Binary> {
    let address = deps.api.addr_validate(&address).unwrap();
    let current_cooldown = COOLDOWNS
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    to_binary(&CooldownResponse { current_cooldown })
}
