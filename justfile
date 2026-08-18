set shell := ["bash", "-uc"]

[private]
default:
  just --list

build:
    cargo build --all-features
doc:
    cargo doc --all-features --no-deps --open
test:
    cargo test --all-features


egs: eg-cards eg-bevy eg-peek eg-weighted
eg-cards:
    cargo run --example cards --features=cards
eg-bevy:
    cargo run --example bevy --features "bevy cards rand"
eg-peek:
    cargo run --example peek --features=cards
eg-weighted:
    cargo run --example weighted
