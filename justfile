default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

clippy:
    cargo clippy --no-default-features -- -D warnings

clippy-all:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --no-default-features --no-fail-fast

test-all:
    cargo test --all-targets --all-features --no-fail-fast
    cargo test --doc --all-features

audit:
    cargo generate-lockfile
    cargo audit

check: fmt-check clippy test audit

check-all: fmt-check clippy-all test-all audit
