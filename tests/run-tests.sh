#!/bin/bash
TEST_TYPE=$1
set -eu

set +u
OPTIONAL_KEY=""
if [ ! -z $2 ];
    then OPTIONAL_KEY="$2"
fi
set -u

# Run test entry point script
docker exec gravity_test_instance --env USE_LOCAL_ARTIFACTS=${USE_LOCAL_ARTIFACTS:-0} /bin/sh -c "pushd /gravity/ && tests/container-scripts/integration-tests.sh $TEST_TYPE"
