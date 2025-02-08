check:
    cargo fmt --check
    cargo clippy --all-features --quiet -- -Dwarnings
    cargo test run --quiet --all-features

fix:
    cargo fmt
    cargo clippy --fix --allow-dirty
    cargo update
