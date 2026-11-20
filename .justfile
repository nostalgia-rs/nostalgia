test:
    cargo fmt
    cargo clippy
    cargo test

run:
    cargo run

build:
    cargo build

release:
    cargo build --release