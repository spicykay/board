# Board — local automation. All tooling (cargo, node, just) is provided by Hermit
# under ./bin, which is prepended to PATH for every recipe below.

set shell := ["bash", "-uc"]
export PATH := justfile_directory() + "/bin:" + env_var('PATH')

# Show available recipes.
default:
    @just --list

# Install JavaScript dependencies.
setup:
    npm install

# Run the desktop app in dev mode (hot-reloading UI + Rust).
dev: setup
    npm run tauri dev

# Build the production desktop app bundle.
build: setup
    npm run tauri build

# Build the release `board` CLI binary -> target/release/board.
cli-build:
    cargo build --release -p board-cli

# Install the `board` CLI onto your PATH (~/.cargo/bin or cargo install root).
cli-install:
    cargo install --path crates/cli

# Run the board CLI through cargo, e.g. `just cli list` or `just cli new --title Hi`.
cli *ARGS:
    cargo run -q -p board-cli -- {{ARGS}}

# Run the Rust test suite.
test:
    cargo test

# Type-check Rust workspace and the TypeScript frontend.
check: setup
    cargo check --workspace
    npx tsc --noEmit

# Format Rust and frontend sources.
fmt:
    cargo fmt
    npx prettier --write "src/**/*.{ts,tsx,css}" "*.html"

# Lint Rust with clippy (warnings are errors).
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run the full local check suite (what CI would run).
ci: check lint test

# Remove build artifacts.
clean:
    cargo clean
    rm -rf dist node_modules
