#!/bin/bash
set -eux

# this directy of this script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DOCKERFOLDER=$DIR/dockerfile
REPOFOLDER=$DIR/..

#docker system prune -a -f

# By default we want to do a clean build, but for faster development `USE_LOCAL_ARTIFACTS=1` can
# be set in which case binaries that reuse local artifacts will be placed into the docker image
if [[ "${USE_LOCAL_ARTIFACTS:-0}" -eq "0" ]]; then
    # change our directory so that the git archive command works as expected
    pushd $REPOFOLDER

    # don't compress to `tar.gz`, because it adds more build time and
    # will be uncompressed anyway when added to the container
    git archive --format=tar -o $DOCKERFOLDER/gravity.tar --prefix=gravity/ HEAD
else
    # getting the `test-runner` binary with the x86_64-linux-musl, because the tests will be running on linix
    pushd $REPOFOLDER/orchestrator/test_runner && PATH=$PATH:$HOME/.cargo/bin CC_x86_64_unknown_linux_musl="x86_64-linux-musl-gcc" cargo build --release --all --target=x86_64-unknown-linux-musl
    # getting the `gravity` binary. `GOBIN` is set so that it is placed under `/dockerfile`.
    # This will be moved to its normal place by the `Dockerfile`.
    pushd $REPOFOLDER/module/ &&
        PATH=$PATH:/usr/local/go/bin GOPROXY=https://proxy.golang.org go mod vendor &&
        PATH=$PATH:/usr/local/go/bin GOBIN=$DOCKERFOLDER make build-linux-amd64

    # build npm artifacts
    pushd $REPOFOLDER/solidity/ &&
    HUSKY_SKIP_INSTALL=1 npm ci
    npm run typechain

    # compress binaries
    pushd $DOCKERFOLDER
    tar -cvf gravity.tar gravity

    # clean
    rm gravity
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

# clear gravity archive
rm gravity.tar