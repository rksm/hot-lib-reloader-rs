default:
    just --list

# -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
# Dev tasks / tests

test dir=".":
    cd {{ dir }} && \
      cargo nextest run --workspace --all-features --no-tests warn && \
      cargo test --doc --workspace

lint dir=".":
    cd {{ dir }} && \
      cargo clippy --all-features -- -D warnings

fmt dir=".":
    cd {{ dir }} && \
      cargo fmt --all

fmt-check dir=".":
    cd {{ dir }} && \
      cargo fmt --all -- --check

check dir=".": (fmt-check dir) (test dir) readme-check

check-all:
    #!/usr/bin/env bash
    set -e
    for dir in $(scripts/rust-crates.py list-workspaces); do
        echo "Checking $dir"
        if [[ "$dir" == "examples/bevy" ]] || \
           [[ "$dir" == "examples/hot-egui" ]] || \
           [[ "$dir" == "examples/hot-iced" ]] || \
           [[ "$dir" == "examples/nannou-vector-field" ]]; then
            continue
        fi
        just check $dir
    done
    nix develop .#gui -c just check examples/bevy
    nix develop .#gui -c just check examples/hot-egui
    nix develop .#gui -c just check examples/hot-iced
    nix develop .#gui -c just check examples/nannou-vector-field

run-minimal:
    cd examples/minimal && just run

run-minimal-test:
    cd examples/minimal && cargo test -- --no-capture


# -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
# Release

readme:
    cargo rdme --force

readme-check dir=".":
    cd {{ dir }} && \
      cargo rdme --check
