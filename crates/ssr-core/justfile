check:
    cargo fmt --check
    cargo clippy --all-features --quiet -- -Dwarnings
    cargo semver-checks --all-features
    cargo test run --quiet --all-features

publish: check
    cargo publish

fix:
    cargo fmt
    cargo clippy --fix --allow-dirty
    cargo update
