#!/usr/bin/env bash
#
# Board — one-shot installer/bootstrapper.
#
# Downloads the pinned toolchain (Rust, Node, Just) via Hermit if it isn't already
# present, then installs JS dependencies. Safe to re-run; everything is idempotent.
#
#   ./install.sh
#
set -euo pipefail

# Always operate from the repo root (the dir this script lives in).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
cd "$ROOT"

info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || die "curl is required but not found on PATH."

# 1. Ensure the Hermit launcher exists.
#
# ./bin/hermit is committed and self-bootstrapping: on first run it downloads the
# real Hermit binary into Hermit's cache. If someone fetched the sources without the
# ./bin stubs, install Hermit and regenerate them.
if [ ! -x ./bin/hermit ]; then
  info "Hermit launcher missing; installing Hermit and initializing ./bin ..."
  export HERMIT_STATE_DIR="${HERMIT_STATE_DIR:-${HOME}/.cache/hermit}"
  curl -fsSL https://github.com/cashapp/hermit/releases/download/stable/install.sh | bash
  # The installer drops `hermit` into Hermit's bin; locate it and init this repo.
  HERMIT_BIN="$(command -v hermit || echo "${HERMIT_STATE_DIR}/bin/hermit")"
  [ -x "$HERMIT_BIN" ] || die "Hermit install did not produce a usable 'hermit' binary."
  "$HERMIT_BIN" init "$ROOT"
fi

# 2. Download + install the pinned packages for this environment (Rust, Node, Just).
#    The first invocation of ./bin/hermit also self-installs the Hermit runtime.
info "Installing pinned toolchain (Rust, Node, Just) via Hermit ..."
./bin/hermit install

# 3. Bootstrap project dependencies (npm install, via the Just recipe).
info "Installing JavaScript dependencies ..."
./bin/just setup

info "Done. Next steps:"
cat <<'EOF'

  Activate the toolchain (optional, adds ./bin to PATH):
      . bin/activate-hermit

  Run the desktop app (hot reload):
      just dev          # or: ./bin/just dev

  Build & install the `board` CLI:
      just cli-install  # or: ./bin/just cli-install

EOF
