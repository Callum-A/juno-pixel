#!/bin/bash

DEFAULT_DEV_ADDRESS="juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y"

echo "Provisioning - juno address $DEFAULT_DEV_ADDRESS"

# pinched and adapted from whoami/DA0DA0
IMAGE_TAG=${2:-"v7.0.0-alpha"}
CONTAINER_NAME="juno_pixel"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
CHAIN_ID='testing'
RPC='http://localhost:26657/'
TXFLAG="--gas-prices 0.1$DENOM --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"
BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # should mirror mainnet

echo "Building $IMAGE_TAG"
echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

# kill any orphans
docker kill $CONTAINER_NAME
docker volume rm -f junod_data

if [ $(arch) = "arm64" ]; then
  OARCH="-arm64"
  AARCH="-aarch64"
fi

set -e

# run junod setup script
docker run --rm -d --name $CONTAINER_NAME \
    -e STAKE_TOKEN=$DENOM \
    -e GAS_LIMIT="$GAS_LIMIT" \
    -e UNSAFE_CORS=true \
    -p 1317:1317 -p 26656:26656 -p 26657:26657 \
    --mount type=volume,source=junod_data,target=/root \
    ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $DEFAULT_DEV_ADDRESS

# compile
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer$OARCH:0.12.6

docker cp artifacts/juno_pixel.wasm $CONTAINER_NAME:/juno_pixel.wasm

sleep 5

# validator addr
VALIDATOR_ADDR=$($BINARY keys show validator --address)
echo "Validator address:"
echo $VALIDATOR_ADDR

# check balance
BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR)
echo "Pre-store balance:"
echo $BALANCE_1

# default dev user
echo "Address to deploy contracts: $DEFAULT_DEV_ADDRESS"
echo "TX Flags: $TXFLAG"

# upload pixel wasm
CONTRACT_CODE=$($BINARY tx wasm store "/juno_pixel.wasm" --from "validator" $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Juno Pixel Code ID: $CONTRACT_CODE"

# add default test user
echo "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose" | $BINARY keys add test-user --recover --keyring-backend test

PIXEL_INIT='{
  "admin_address": "'"$DEFAULT_DEV_ADDRESS"'",
  "cooldown": 1,
  "end_height": null,
  "width": 10,
  "height": 10
}'
echo "$PIXEL_INIT" | jq .
# instantiate
$BINARY tx wasm instantiate $CONTRACT_CODE "$PIXEL_INIT" \
  --from "validator" --label "Juno Pixel" $TXFLAG  --no-admin

PIXEL_CONTRACT=$($BINARY q wasm list-contract-by-code $CONTRACT_CODE --output json | jq -r '.contracts[-1]')

# Query the initial grid (first chunk)
GET_GRID='{ "get_chunk": { "x": 0, "y": 0 } }'

GRID=$($BINARY query wasm contract-state smart "$PIXEL_CONTRACT" "$GET_GRID" --output json | jq -r .data.grid)
echo "$GRID" | jq .

# Draw
DRAW='{
  "draw": {
    "chunk_x": 0,
    "chunk_y": 0,
    "x": 0,
    "y": 0,
    "color": "red"
  }
}'
echo "$DRAW" | jq .
$BINARY tx wasm execute "$PIXEL_CONTRACT" "$DRAW" --from test-user $TXFLAG

# Load the first chunk should now show red pixel by dev user
GRID=$($BINARY query wasm contract-state smart "$PIXEL_CONTRACT" "$GET_GRID" --output json | jq -r .data.grid[0][0])
echo "$GRID" | jq .

# Wait a block before drawing again
sleep 6

# Draw
DRAW='{
  "draw": {
    "chunk_x": 0,
    "chunk_y": 0,
    "x": 1,
    "y": 0,
    "color": "yellow"
  }
}'
echo "$DRAW" | jq .
$BINARY tx wasm execute "$PIXEL_CONTRACT" "$DRAW" --from test-user $TXFLAG

# Show that this pixel is yellow
GRID=$($BINARY query wasm contract-state smart "$PIXEL_CONTRACT" "$GET_GRID" --output json | jq -r .data.grid[1][0])
echo "$GRID" | jq .

echo "NEXT_PUBLIC_PIXEL_ADDRESS=$PIXEL_CONTRACT"


