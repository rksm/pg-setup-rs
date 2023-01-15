default:
    just --list

example *args='':
    cargo run --example {{args}}
