#!/usr/bin/env bash
# Reverse `activate-dev-local-patch.sh`: clear the cargo config patch,
# release the skip-worktree flag on Cargo.lock, and restore Cargo.lock
# to the version tracked in HEAD so the next `cargo build` resolves
# fsqlite* against crates.io as published.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [[ -f .cargo/config.toml ]]; then
  rm -f .cargo/config.toml
  echo "removed .cargo/config.toml"
fi

if git ls-files -v Cargo.lock | grep -q '^S'; then
  git update-index --no-skip-worktree Cargo.lock
  echo "Cargo.lock: skip-worktree off"
fi

git checkout -- Cargo.lock
echo "Cargo.lock: restored from HEAD"
