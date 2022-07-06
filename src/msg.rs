use crate::state::{Color, PixelInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin_address: String,
    pub cooldown: u64,
    pub end_height: Option<u64>,
    pub width: u64,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Draw { x: u64, y: u64, color: Color },
    UpdateAdmin { new_admin_address: String },
    UpdateCooldown { new_cooldown: u64 },
    UpdateEndHeight { new_end_height: Option<u64> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGrid {},
    GetConfig {},
    GetDimensions {},
    GetCooldown { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GridResponse {
    pub grid: Vec<Vec<PixelInfo>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CooldownResponse {
    pub current_cooldown: u64,
}
