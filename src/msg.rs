use crate::state::Color;
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
    UpdateAdmin { new_admin: String },
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

// We define a custom struct for each query response
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CountResponse {
//     pub count: i32,
// }
