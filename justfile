default:
    just --list

test:
    cargo nextest run --workspace --all-features
    cargo test --doc

lint:
    cargo clippy --all-features -- -D warnings

fmt-check:
    cargo fmt --all -- --check

check: fmt-check lint test

fmt:
    cargo fmt --all
