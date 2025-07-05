# Default task: run all checks and tests
default: check test

# Run tests and automatically re-run on file changes.
# Requires `cargo install cargo-watch`.
watch:
    cargo watch -x "test -- --nocapture"

# Check the project for compilation errors
check:
    cargo check

# Run clippy for linting and style checks
lint: check
    cargo clippy -- -D warnings

# Run all tests
test: lint
    #!/usr/bin/env bash
    set -e
    # cargo test -- --nocapture
    RUST_BACKTRACE=1 cargo test --workspace
    #cargo insta test --review
    cargo insta test

# Review and accept/reject any snapshot changes from `insta`.
insta-review:
    cargo insta review


# Run the application with given arguments.
# Example: `just run -- --file README.md --help`
run *args:
    cargo run -- {{args}}

# Generate a directory snapshot for the project
snapshot:
    #!/usr/bin/env bash
    project_name="$(basename "${PWD%.git}")"
    snapshot_filename=".${project_name}_repo_snapshot.md"
    RIPGREP_CONFIG_PATH="{{invocation_directory()}}/.ripgreprc" dir2prompt > "${snapshot_filename}"
    wc -c "${snapshot_filename}"
