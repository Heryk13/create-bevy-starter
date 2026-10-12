# apply clippy fixes and format the code
fix:
    cargo clippy --all-targets --fix --allow-dirty
    cargo fmt

# check formatting and lints (tests included), and run the tests
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# run the tests only
test:
    cargo test

# run the CLI, passing on its arguments: just run <name> <folder>
run *args:
    cargo run -- {{args}}
