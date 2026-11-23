test:
    cargo fmt
    cargo clippy
    cargo nextest run

coverage:
    cargo llvm-cov --all-targets --fail-under-lines 95

coverage-html:
    cargo llvm-cov --all-targets --fail-under-lines 95 --html --open

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
    cargo clippy --all-targets -- -D missing_docs
    cargo doc --no-deps --document-private-items --open

run:
    cargo run

build:
    cargo build

release:
    cargo build --release