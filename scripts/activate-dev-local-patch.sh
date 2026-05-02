#!/usr/bin/env bash
# Activate the sibling-path frankensqlite/fastmcp_rust patches for local
# co-development. Cargo.lock is marked skip-worktree so dev-local rewrites
# (stripping registry source/checksum lines from patched crates) are not
# accidentally staged or committed. Pair with `deactivate-dev-local-patch.sh`.
#
# Usage:
#   scripts/activate-dev-local-patch.sh           # frankensqlite only
#   scripts/activate-dev-local-patch.sh --fastmcp # also enable fastmcp_rust
#
# Idempotent: safe to re-run.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [[ ! -f scripts/dev-local-frankensqlite.toml ]]; then
  echo "error: scripts/dev-local-frankensqlite.toml not found" >&2
  exit 1
fi

if [[ ! -d ../frankensqlite/crates/fsqlite ]]; then
  echo "error: sibling frankensqlite checkout not found at ../frankensqlite" >&2
  echo "       run \`git clone https://github.com/Dicklesworthstone/frankensqlite ../frankensqlite\` first" >&2
  exit 1
fi

mkdir -p .cargo

if [[ "${1:-}" == "--fastmcp" ]]; then
  # Strip the leading `# ` from the fastmcp_rust block so those entries activate too.
  sed -E 's/^# (fastmcp-[a-z]+ +=)/\1/' scripts/dev-local-frankensqlite.toml > .cargo/config.toml
  echo "activated dev-local patch (frankensqlite + fastmcp_rust)"
else
  cp scripts/dev-local-frankensqlite.toml .cargo/config.toml
  echo "activated dev-local patch (frankensqlite only — pass --fastmcp to also enable fastmcp_rust)"
fi

git update-index --skip-worktree Cargo.lock
echo "Cargo.lock: skip-worktree on (cargo rewrites are now invisible to git)"
