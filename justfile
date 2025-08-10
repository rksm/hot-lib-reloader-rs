default:
    just --list

test dir=".":
    cd {{ dir }} && \
      cargo nextest run --workspace --all-features --no-tests warn && \
      cargo test --doc --workspace

lint dir=".":
    cd {{ dir }} && \
      cargo clippy --all-features -- -D warnings

fmt:
    just fmt-dir "."
    just fmt-dir "examples/minimal"

[private]
fmt-dir dir=".":
    cd {{ dir }} && \
      cargo fmt --all

fmt-check:
    just fmt-check "."
    just fmt-check "examples/minimal"

[private]
fmt-check-dir dir=".":
    cd {{ dir }} && \
      cargo fmt --all -- --check

check dir=".": (fmt-check-dir dir) (test dir)

check-all:
    just check examples/minimal
    # examples/all-options/lib
    # examples/bevy/components
    # examples/bevy/systems
    # examples/hot-egui/lib
    # examples/hot-iced/lib
    # examples/nannou-vector-field/lib
    # examples/nannou-vector-field/nannou_dynamic
    # examples/reload-events/lib
    # examples/reload-feature/lib
    # examples/serialized-state/lib
    # examples/two-libs/lib1
    # examples/two-libs/lib2
    # macro
    # macro-no-mangle-if-debug
    # tests/lib_for_testing

run-minimal:
    cd examples/minimal && just run

run-minimal-test:
    cd examples/minimal && cargo test -- --no-capture
