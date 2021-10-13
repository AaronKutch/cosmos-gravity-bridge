#!/bin/bash
set -eu

echo "running cosmos-gravity-bridge"

# Initial dir
CURRENT_WORKING_DIR=$(pwd)
# Name of the network to bootstrap
CHAINID="testchain"
# Name of the gravity artifact
GRAVITY=gravity
# The name of the gravity node
GRAVITY_NODE_NAME="gravity"
# The address to run gravity node
GRAVITY_HOST="0.0.0.0"
# The port of the gravity gRPC
GRAVITY_GRPC_PORT="9090"
# Home folder for gravity config
GRAVITY_HOME="$CURRENT_WORKING_DIR/$CHAINID/$GRAVITY_NODE_NAME"
# Home flag for home folder
GRAVITY_HOME_FLAG="--home $GRAVITY_HOME"
# Gravity mnemonic used for orchestrator signing of the transactions (orchestrator_key.json file)
GRAVITY_ORCHESTRATOR_MNEMONIC=$(jq -r .mnemonic $GRAVITY_HOME/orchestrator_key.json)

# Gravity chain demons
STAKE_DENOM="stake"

# The ETH key used for orchestrator signing of the transactions
ETH_ORCHESTRATOR_PRIVATE_KEY=c40f62e75a11789dbaf6ba82233ce8a52c20efb434281ae6977bb0b3a69bf709

ETH_CONTRACT_ADDRESS=0x8497452388c710Ebecd09adBF018C41f1Fe69876

ETH_ADDRESS=""

# ------------------ Run gravity ------------------

echo "Starting $GRAVITY_NODE_NAME"
$GRAVITY $GRAVITY_HOME_FLAG start --pruning=nothing &

echo "Waiting $GRAVITY_NODE_NAME to launch gRPC $GRAVITY_GRPC_PORT..."

while ! timeout 1 bash -c "</dev/tcp/$GRAVITY_HOST/$GRAVITY_GRPC_PORT"; do
  sleep 1
done

echo "$GRAVITY_NODE_NAME launched"

#-------------------- Run orchestrator --------------------

echo "Starting orchestrator"

gbt orchestrator --cosmos-phrase="$GRAVITY_ORCHESTRATOR_MNEMONIC" \
             --ethereum-key="$ETH_ORCHESTRATOR_PRIVATE_KEY" \
             --cosmos-grpc="http://$GRAVITY_HOST:$GRAVITY_GRPC_PORT/" \
             --ethereum-rpc="$ETH_ADDRESS" \
             --fees="1$STAKE_DENOM" \
             --gravity-contract-address="$ETH_CONTRACT_ADDRESS"