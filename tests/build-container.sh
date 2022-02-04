#!/bin/bash
set -eux

# this directy of this script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DOCKERFOLDER=$DIR/dockerfile
REPOFOLDER=$DIR/..

# By default we want to do a clean build, but for faster development `USE_LOCAL_ARTIFACTS=1` can
# be set in which case binaries that reuse local artifacts will be placed into the docker image
if [[ "${USE_LOCAL_ARTIFACTS:-0}" -eq "0" ]]; then
    # change our directory so that the git archive command works as expected
    pushd $REPOFOLDER

    #docker system prune -a -f
    # Build base container
    #git archive --format=tar.gz -o $DOCKERFOLDER/gravity.tar.gz --prefix=gravity/ HEAD

    git archive --format=tar.gz -o $DOCKERFOLDER/gravity.tar.gz --prefix=gravity/ HEAD
else
    # getting the `test-runner` binary
    pushd $REPOFOLDER/orchestrator/test_runner && PATH=$PATH:$HOME/.cargo/bin cargo build --all --release
    # getting the `gravity` binary. `GOBIN` is set so that it is placed under `/dockerfile`.
    # We check for its existence and rename it to prevent confusion.
    # This will be moved to its normal place by the `Dockerfile`.
    pushd $REPOFOLDER/module/ &&
        PATH=$PATH:/usr/local/go/bin GOPROXY=https://proxy.golang.org make &&
        PATH=$PATH:/usr/local/go/bin GOBIN=$REPOFOLDER/tests/dockerfile make install &&
        mv $REPOFOLDER/tests/dockerfile/gravity $REPOFOLDER/tests/dockerfile/go_gravity_bin

    # TODO I should be able to use local npm results but am encountering errors
    #pushd $REPOFOLDER/solidity/ && npm ci
    #pushd $REPOFOLDER/solidity/
    #HUSKY_SKIP_INSTALL=1 npm install
    #npm run typechain
    #npx hardhat typechain

    # change our directory so that the git archive command works as expected
    pushd $REPOFOLDER

    #docker system prune -a -f
    # Build base container
    #git archive --format=tar.gz -o $DOCKERFOLDER/gravity.tar.gz --prefix=gravity/ HEAD

    # because `--add-file` is not available except in very recent versions of `git`, manually append binaries
    git archive --format=tar -o $DOCKERFOLDER/gravity.tar --prefix=gravity/ HEAD
    tar --append --file=$DOCKERFOLDER/gravity.tar --transform='s,orchestrator/target/release/test-runner,gravity/orchestrator/target/release/test-runner,' $REPOFOLDER/orchestrator/target/release/test-runner
    # this is the go binary
    tar --append --file=$DOCKERFOLDER/gravity.tar --transform='s,tests/dockerfile/go_gravity_bin,gravity/tests/dockerfile/go_gravity_bin,' $REPOFOLDER/tests/dockerfile/go_gravity_bin
    # solidity artifacts
    #tar --append --file=$DOCKERFOLDER/gravity.tar --transform='s,^solidity/artifacts/,gravity/solidity/artifacts/,' $REPOFOLDER/solidity/artifacts
    #tar --append --file=$DOCKERFOLDER/gravity.tar --transform='s,^solidity/typechain/,gravity/solidity/typechain/,' $REPOFOLDER/solidity/typechain
    gzip -f $DOCKERFOLDER/gravity.tar
fi


pushd $DOCKERFOLDER

# setup for Mac M1 Compatibility 
PLATFORM_CMD=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ -n $(sysctl -a | grep brand | grep "M1") ]]; then
       echo "Setting --platform=linux/amd64 for Mac M1 compatibility"
       PLATFORM_CMD="--platform=linux/amd64"; fi
fi
docker build -t gravity-base $PLATFORM_CMD . --build-arg use_local_artifacts=${USE_LOCAL_ARTIFACTS:-0}
