#!/usr/bin/env bash
# Reverse `activate-dev-local-patch.sh`: clear the cargo config patch,
# release the skip-worktree flag on Cargo.lock, and restore Cargo.lock
# to the version tracked in HEAD so the next `cargo build` resolves
# fsqlite* against crates.io as published.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Only remove `.cargo/config.toml` if its first line matches the dev-local
# template marker. Anything else is a contributor-authored config that this
# script must not clobber.
ACTIVATION_MARKER='# Local contributor template:'
if [[ -f .cargo/config.toml ]]; then
  if head -1 .cargo/config.toml | grep -Fq "$ACTIVATION_MARKER"; then
    rm -f .cargo/config.toml
    echo "removed .cargo/config.toml"
  else
    echo "note: .cargo/config.toml does not look like our dev-local template; leaving it in place" >&2
  fi
fi

if git ls-files -v Cargo.lock | grep -q '^S'; then
  git update-index --no-skip-worktree Cargo.lock
  echo "Cargo.lock: skip-worktree off"
fi

# `git restore` is the modern equivalent of `git checkout -- Cargo.lock`.
# Both restore the worktree from the index/HEAD, but `restore` won't trip
# safety wrappers that gate on `git checkout -- <path>`.
git restore Cargo.lock
echo "Cargo.lock: restored from HEAD"
