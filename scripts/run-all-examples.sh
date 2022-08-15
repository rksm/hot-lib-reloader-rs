#!/usr/bin/env bash

function run_example {
    dir=$1
    pushd $dir
    echo "--------- $dir ------------"
    cargo runcc -c
    popd
}

run_example examples/minimal
run_example examples/reload-feature
run_example examples/all-options

run_example examples/reload-events
run_example examples/serialized-state
run_example examples/bevy
