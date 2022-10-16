#!/bin/bash
# Starts the Ethereum testnet chain in the background

set -ex
TEST_TYPE=$1
ALCHEMY_ID=$2
# GETH and TEST_TYPE may be unbound, don't `set -u`

# Starts a hardhat RPC backend that is based off of a fork of Ethereum mainnet. This is useful in that we take
# over the account of a major Uniswap liquidity provider and from there we can test many things that are infeasible
# to do with a Geth backend, simply becuase reproducting that state on our testnet would be far too complex to consider
# The tradeoff here is that hardhat is an ETH dev environment and not an actual ETH implementation, as such the outputs
# may be different. These two tests have different fork block heights they rely on
if [[ $TEST_TYPE == *"ARBITRARY_LOGIC"* ]]; then
    export ALCHEMY_ID=$ALCHEMY_ID
    pushd /gravity/solidity
    npm run solidity_test_fork &
    popd
elif [[ $TEST_TYPE == *"RELAY_MARKET"* ]]; then
    export ALCHEMY_ID=$ALCHEMY_ID
    pushd /gravity/solidity
    npm run evm_fork &
    popd
# This starts a hardhat test environment with no pre-seeded state, faster to run, not accurate
elif [[ ! -z "$HARDHAT" ]]; then
    pushd /gravity/solidity
    npm run evm &
    popd
# This starts the Geth backed testnet with no pre-seeded in state.
# Geth is what we run in CI and in general, but developers frequently
# perfer a faster experience provided by HardHat, also Mac's do not
# work correctly with the Geth backend, there is some issue where the Docker VM on Mac platforms can't get
# the right number of cpu cores and Geth goes crazy consuming all the processing power, on the other hand
# hardhat doesn't work for some tests that depend on transactions waiting for blocks, so Geth is the default
else
    # To make a custom genesis file for `go-opera`, comment out the normal `opera`
    # command below and edit `test_runner/src/main.rs` by changing `MINER_PRIVATE_KEY`
    # to use 0x163F5F0F9A621D72FEDD85FFCA3D08D131AB4E812181E0D30FFD1C885D20AAC7
    # and uncommenting the special `send_eth_bulk` that sends tokens to the address we
    # want to use. Uncomment the other `opera` command below which will use Fantom's
    # default genesis. Then, run `USE_LOCAL_ARTIFACTS=1 bash tests/all-up-test.sh NO_SCRIPTS`
    # and get a command prompt to the running container. In the container run
    # `bash /gravity/tests/container-scripts/all-up-test-internal.sh 4` and wait for the panic
    # "sent eth to default address" (or for some reason the test runner can hang, look at
    # `opera.log` to see if the transaction has happened and then kill the test runner).
    # Then in the container `pkill opera` and run
    # `opera --datadir /opera_datadir/ export genesis /gravity/tests/assets/test_genesis.g --export.evm.mode=ext-mpt`
    # which will convert the state of the testchain up to that point into a new genesis that we
    # use for normal runs. Commit the `test_genesis.g` and undo the other changes.
    opera --fakenet 1/1 \
        --nodiscover \
        --http \
        --http.addr="localhost" \
        --http.port="8545" \
        --http.api="eth,debug,net,admin,web3,personal,txpool,ftm,dag" \
        --datadir="/opera_datadir" &> /opera.log &

    # The fakenet chain id is 4003, which is different from the production id of 250
    #opera --genesis="/gravity/tests/assets/test_genesis.g" \
    #    --genesis.allowExperimental=true \
    #    --nodiscover \
    #    --http \
    #    --http.addr="localhost" \
    #    --http.port="8545" \
    #    --http.api="eth,debug,net,admin,web3,personal,txpool,ftm,dag" \
    #    --datadir="/opera_datadir" &> /opera.log &

    echo "waiting for go-opera to come online"
    until $(curl --output /dev/null --fail --silent --header "content-type: application/json" --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}' http://localhost:8545); do
        sleep 1
    done
    # go-opera takes a few seconds to sync
    echo "waiting for go-opera to sync"
    until [ "$(curl -s --header "content-type: application/json" --data '{"id":1,"jsonrpc":"2.0","method":"eth_syncing","params":[]}' http://localhost:8545)" == '{"jsonrpc":"2.0","id":1,"result":false}' ]; do
        sleep 1
    done

    # send from dev account (priv 0x163F5F0F9A621D72FEDD85FFCA3D08D131AB4E812181E0D30FFD1C885D20AAC7/pub 0x239fA7623354eC26520dE878B52f13Fe84b06971) to 0xBf660843528035a5A4921534E156a27e64B231fE
    curl -s --header "content-type: application/json" --data '{"id":12,"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0xf871808506fc23ac00825dc094bf660843528035a5a4921534e156a27e64b231fe8b52b7d2dcc80cd2e400000080821f6aa020bac4e3944d6b12075eee0e58ec9e40270a9a977feba34362190a825575f550a07272d97392a1a7d0bda2a0bb62da49b45275f873259a16e463603a60abf9a5cf"]}' http://localhost:8545

    echo "waiting for transaction to account 0xBf6608..."
    until [ "$(curl -s --header "content-type: application/json" --data '{"id":1,"jsonrpc":"2.0","method":"eth_getBalance","params":["0xBf660843528035a5A4921534E156a27e64B231fE","latest"]}' http://localhost:8545)" == '{"jsonrpc":"2.0","id":1,"result":"0x52b7d2dcc80cd2e4000000"}' ]; do
        sleep 1
    done
fi
