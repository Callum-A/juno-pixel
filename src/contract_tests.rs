#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::ExecuteMsg::{Draw, UpdateAdmin, UpdateCooldown, UpdateEndHeight};
    use crate::msg::{ChunkResponse, CooldownResponse, InstantiateMsg, QueryMsg};
    use crate::state::{Config, Dimensions, PixelInfo};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr, Deps, Env};

    pub const ADDR1: &str = "addr1";
    pub const ADDR2: &str = "addr2";

    fn query_config(deps: Deps, env: Env) -> Config {
        let msg = QueryMsg::GetConfig {};
        let bin = query(deps, env, msg).unwrap();
        from_binary(&bin).unwrap()
    }

    fn query_chunk(deps: Deps, env: Env, x: u64, y: u64) -> ChunkResponse {
        let msg = QueryMsg::GetChunk { x, y };
        let bin = query(deps, env, msg).unwrap();
        from_binary(&bin).unwrap()
    }

    fn query_dimensions(deps: Deps, env: Env) -> Dimensions {
        let msg = QueryMsg::GetDimensions {};
        let bin = query(deps, env, msg).unwrap();
        from_binary(&bin).unwrap()
    }

    fn query_cooldown(deps: Deps, env: Env, address: String) -> CooldownResponse {
        let msg = QueryMsg::GetCooldown { address };
        let bin = query(deps, env, msg).unwrap();
        from_binary(&bin).unwrap()
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        let current_height = env.block.height;

        // Invalid end height
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: Some(current_height - 1),
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        // Valid height
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: Some(current_height + 1),
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // No height
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: None,
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Check dimensions are setup correctly
        let dimensions = query_dimensions(deps.as_ref(), env.clone());
        assert_eq!(
            dimensions,
            Dimensions {
                width: 100,
                height: 100
            }
        );
        let grid = query_chunk(deps.as_ref(), env, 0, 0);
        assert_eq!(grid.grid.len(), 32);
        assert_eq!(grid.grid[0].len(), 32);
    }

    #[test]
    fn test_draw() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info(ADDR1, &[]);
        let start_height = env.block.height;
        let end_height = start_height + 200;

        // Instantiate with ADDR1 as admin
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: Some(end_height),
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Query cooldown when we have not drawn, should return 0
        let cooldown = query_cooldown(deps.as_ref(), env.clone(), ADDR1.to_string());
        assert_eq!(cooldown.current_cooldown, 0);

        // Try and draw with invalid color code (outside 0-15 range)
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 0,
            y: 0,
            color: 16, // Unrecognised
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        // Try and draw with invalid dimensions (within the chunk)
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 32,
            y: 32,
            color: 3, // Black
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        // Try and draw with invalid dimensions (chunk dimensions)
        let msg = Draw {
            chunk_x: 100,
            chunk_y: 100,
            x: 16,
            y: 16,
            color: 3, // Black
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        // Successful draw ADDR1
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 0,
            y: 0,
            color: 3, // Black
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        // Successful draw ADDR2
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 1,
            y: 0,
            color: 5, // Red
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR2, &[]), msg).unwrap();

        // Query cooldown should be start_height + 30
        let cooldown = query_cooldown(deps.as_ref(), env.clone(), ADDR1.to_string());
        assert_eq!(cooldown.current_cooldown, start_height + 30);
        // Query cooldown should be start_height + 30
        let cooldown = query_cooldown(deps.as_ref(), env.clone(), ADDR2.to_string());
        assert_eq!(cooldown.current_cooldown, start_height + 30);

        let grid = query_chunk(deps.as_ref(), env.clone(), 0, 0);
        assert_eq!(
            grid.grid[0][0],
            PixelInfo {
                color: 3, // Black
                painter: Some(Addr::unchecked(ADDR1.to_string()))
            }
        );
        assert_eq!(
            grid.grid[0][1],
            PixelInfo {
                color: 5, // Red
                painter: Some(Addr::unchecked(ADDR2.to_string()))
            }
        );

        // Try and draw prior to cooldown, will error
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 0,
            y: 0,
            color: 3, // Black
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR2, &[]), msg).unwrap_err();

        // Override existing color after cooldown
        env.block.height = start_height + 30;
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 0,
            y: 0,
            color: 5, // Red
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR2, &[]), msg).unwrap();

        // Query cooldown should be start_height + 30 + 30, as we have now drawn twice
        let cooldown = query_cooldown(deps.as_ref(), env.clone(), ADDR2.to_string());
        assert_eq!(cooldown.current_cooldown, start_height + 30 + 30);

        let grid = query_chunk(deps.as_ref(), env.clone(), 0, 0);
        assert_eq!(
            grid.grid[0][0],
            PixelInfo {
                color: 5, // Red
                painter: Some(Addr::unchecked(ADDR2.to_string()))
            }
        );

        // Try and draw after the end_height
        env.block.height = end_height + 1;
        let msg = Draw {
            chunk_x: 0,
            chunk_y: 0,
            x: 0,
            y: 0,
            color: 10, // Green
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();

        // Grid unchanged
        let grid = query_chunk(deps.as_ref(), env, 0, 0);
        assert_eq!(
            grid.grid[0][0],
            PixelInfo {
                color: 5, // Red
                painter: Some(Addr::unchecked(ADDR2.to_string()))
            }
        );
    }

    #[test]
    fn test_update_admin() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);

        // Instantiate with ADDR1 as admin
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: None,
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Try and update admin as ADDR2, should error
        let msg = UpdateAdmin {
            new_admin_address: ADDR2.to_string(),
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR2, &[]), msg).unwrap_err();

        // Query config, it hasn't changed
        let config = query_config(deps.as_ref(), env.clone());
        assert_eq!(config.admin_address, Addr::unchecked(ADDR1));

        // Update as ADDR1, should succeed
        let msg = UpdateAdmin {
            new_admin_address: ADDR2.to_string(),
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR1, &[]), msg).unwrap();
        let config = query_config(deps.as_ref(), env);
        assert_eq!(config.admin_address, Addr::unchecked(ADDR2));
    }

    #[test]
    fn test_update_cooldown() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);

        // Instantiate with ADDR1 as admin
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: None,
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Try and update cooldown as ADDR2, should error
        let msg = UpdateCooldown { new_cooldown: 15 };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR2, &[]), msg).unwrap_err();
        let config = query_config(deps.as_ref(), env.clone());
        assert_eq!(config.cooldown, 30);

        // Update as ADDR1, should succeed
        let msg = UpdateCooldown { new_cooldown: 15 };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR1, &[]), msg).unwrap();
        let config = query_config(deps.as_ref(), env);
        assert_eq!(config.cooldown, 15);
    }

    #[test]
    fn test_update_end_height() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        let valid_height = env.block.height + 30;
        let invalid_height = env.block.height - 15;

        // Instantiate with ADDR1 as admin
        let msg = InstantiateMsg {
            admin_address: ADDR1.to_string(),
            cooldown: 30,
            end_height: None,
            width: 100,
            height: 100,
        };
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Try and update end height as ADDR2, should error
        let msg = UpdateEndHeight {
            new_end_height: Some(valid_height),
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR2, &[]), msg).unwrap_err();
        let config = query_config(deps.as_ref(), env.clone());
        assert_eq!(config.end_height, None);

        // Update as ADDR1, but invalid height
        let msg = UpdateEndHeight {
            new_end_height: Some(invalid_height),
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR1, &[]), msg).unwrap_err();
        let config = query_config(deps.as_ref(), env.clone());
        assert_eq!(config.end_height, None);

        // Update as ADDR1, valid height
        let msg = UpdateEndHeight {
            new_end_height: Some(valid_height),
        };
        execute(deps.as_mut(), env.clone(), mock_info(ADDR1, &[]), msg).unwrap();
        let config = query_config(deps.as_ref(), env);
        assert_eq!(config.end_height, Some(valid_height));
    }
}
