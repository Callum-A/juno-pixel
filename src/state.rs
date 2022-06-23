use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Admin address, can update config values.
    pub admin_address: Addr,
    /// Number of blocks between user draws, if set to 30 user
    /// must wait 30 blocks before being able to draw again.
    pub cooldown: u64,
    /// Block height the canvas can no longer be drawn on at all.
    /// Optional so if not set it goes on forever.
    pub end_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Dimensions {
    pub width: u64,
    pub height: u64,
}

// TODO: set the colours so they line up with Camel's IDs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    BLACK = 0,
    WHITE = 1,
    RED = 2,
    GREEN = 3,
    BLUE = 4,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DIMENSIONS: Item<Dimensions> = Item::new("dimensions");
pub const GRID: Item<Vec<Vec<Color>>> = Item::new("grid");
pub const COOLDOWNS: Map<&Addr, u64> = Map::new("cooldowns");
