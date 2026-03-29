---
name: br
description: >-
  Local-first issue tracker (beads_rust) for AI agents. Use when tracking tasks,
  managing dependencies, finding ready work, or syncing issues to git via JSONL.
---

<!-- TOC: Critical Rules | Quick Workflow | Essential Commands | bv Integration | Redirect/Worktree | Troubleshooting | References -->

# br — Beads Rust Issue Tracker

> **Non-invasive:** br NEVER runs git commands. Sync and commit are YOUR responsibility.

## Critical Rules for Agents

| Rule | Why |
|------|-----|
| **ALWAYS use `--json`** | Structured output for parsing |
| **NEVER run bare `bv`** | Blocks session in TUI mode |
| **Sync is EXPLICIT** | `br sync --flush-only` after changes |
| **Git is YOUR job** | br only touches `.beads/` directory |
| **No cycles allowed** | `br dep cycles` must return empty |

## Quick Workflow

```bash
# 1. Find work
br ready --json

# 2. Claim it
br update bd-abc123 --status in_progress

# 3. Do work...

# 4. Complete
br close bd-abc123 --reason "Implemented X"

# 5. Sync to git (EXPLICIT!)
br sync --flush-only
git add .beads/ && git commit -m "feat: X (bd-abc123)"
```

## Essential Commands

```bash
# Lifecycle
br init                              # Initialize .beads/
br create "Title" -p 1 -t task       # Create (priority 0-4)
br update <id> --status in_progress  # Claim work
br close <id> --reason "Done"        # Complete
br reopen <id>                       # Reopen if needed

# Querying (always use --json for agents)
br ready --json                      # Actionable work (not blocked)
br list --json                       # All issues
br blocked --json                    # What's blocked
br search "keyword"                  # Full-text search
br show <id> --json                  # Issue details

# Dependencies
br dep add <child> <parent>          # child depends on parent
br dep cycles                        # MUST be empty!
br dep tree <id>                     # Visualize dependencies

# Sync (EXPLICIT - never automatic)
br sync --flush-only                 # DB → JSONL (before git commit)
br sync --import-only                # JSONL → DB (after git pull)

# System
br doctor                            # Health check
br config --list                     # Show configuration
```

## Priority Scale

| Priority | Meaning |
|----------|---------|
| 0 | Critical |
| 1 | High |
| 2 | Medium (default) |
| 3 | Low |
| 4 | Backlog |

## bv Integration

**CRITICAL:** Never run bare `bv` — it launches interactive TUI and blocks.

```bash
# Always use --robot-* flags:
bv --robot-next                      # Single top pick
bv --robot-triage                    # Full triage
bv --robot-plan                      # Parallel execution tracks
bv --robot-insights | jq '.Cycles'   # Check graph health
```

## Agent Mail Coordination

Use bead ID as thread_id for multi-agent coordination:

```python
file_reservation_paths(..., reason="bd-123")
send_message(..., thread_id="bd-123", subject="[bd-123] Starting...")
# Work...
br close bd-123 --reason "Completed"
release_file_reservations(...)
```

## Session Ending Pattern

```bash
git pull --rebase
br sync --flush-only
git add .beads/ && git commit -m "Update issues"
git push
git status  # Verify clean
```

## Anti-Patterns

- Running `br sync` without `--flush-only` or `--import-only`
- Forgetting sync before git commit
- Creating circular dependencies
- Running bare `bv`
- Assuming auto-commit behavior

## Storage

```
.beads/
├── beads.db        # SQLite (primary)
├── issues.jsonl    # Git-friendly export
└── config.yaml     # Optional config
```

## Redirect / Worktree Workflow

### What Redirects Are

A `.beads/redirect` file is a plain text file containing a single path that
points to another `.beads/` (or `_beads/`) directory. When br discovers a
redirect, it follows the chain until it reaches a terminal beads directory.
All reads and writes (SQLite DB, JSONL) happen in the **target** directory,
not the one containing the redirect file.

This lets multiple working trees share one beads store so issues, history,
and dependencies stay unified.

### When to Use

- **Git worktrees** -- the primary use case. Each worktree gets its own
  `.beads/` from git, but you want all of them to operate on the same
  underlying database.
- **PR-only workflows** -- temporary feature branches that should not
  maintain their own beads state.
- **Ephemeral checkouts** -- CI or review checkouts that need read access
  to issues without duplicating the store.

### Setup

Create a `redirect` file inside the worktree's `.beads/` directory. The
path is resolved **relative to the `.beads/` directory itself**, not the
project root.

```bash
# Example: main repo at /code/myproject, worktree at /code/myproject/.worktrees/feat
# The worktree's .beads/ is at .worktrees/feat/.beads/
# From there, ../../../.beads reaches the main repo's .beads/

echo "../../../.beads" > /code/myproject/.worktrees/feat/.beads/redirect
```

Absolute paths also work:

```bash
echo "/code/myproject/.beads" > /code/myproject/.worktrees/feat/.beads/redirect
```

Verify with `br where`:

```bash
cd /code/myproject/.worktrees/feat
br where
# Output shows the target path and "(via redirect from ...)"

br where --json
# { "path": "/code/myproject/.beads", "redirected_from": ".../feat/.beads", ... }
```

### Path Resolution Rules

| Aspect | Behavior |
|--------|----------|
| Relative paths | Resolved from the `.beads/` dir containing the redirect file |
| Absolute paths | Used as-is |
| Target validation | Must be a `.beads` or `_beads` directory that exists on disk |
| Chain following | Redirects can chain (A -> B -> C) up to 10 hops |
| Loop detection | Paths are canonicalized; revisiting any path in the chain is an error |
| Self-redirect (`.`) | Legal -- resolves to the current `.beads/` dir and stops |

### Effect on Commands

With a redirect active, **all br commands** operate on the target directory:

- `br create`, `br update`, `br close` -- write to the target DB and JSONL
- `br sync --flush-only` -- flushes inline (writes already landed in the
  target); typically reports "nothing to export"
- `br list`, `br ready`, `br show` -- read from the target
- `br doctor` -- checks the target store

### JSONL Commits in PR-Gated Repos

Because writes land in the **main repo's** `.beads/` directory, JSONL changes
only appear in `git status` from the main repo, not from the worktree. This
creates a commit-path question for projects that require PRs to merge to `main`.

Recommended approaches:

1. **Treat JSONL as infrastructure** -- commit `.beads/` changes directly to
   `main` from the main repo working tree, outside the PR flow. JSONL updates
   are mechanical bookkeeping, not feature code, so many teams exempt them
   from PR review.

2. **Periodic sync from main** -- after merging a feature PR, run
   `br sync --flush-only` from the main repo on `main` and commit the
   resulting JSONL changes as a separate housekeeping commit.

3. **Dedicated beads-sync branch** -- configure `br config set sync.branch
   beads-sync` and commit JSONL to that branch, then merge it to `main`
   through your normal PR process.

### Quick Setup Recipe

```bash
# 1. Create worktree
git worktree add .worktrees/feat -b feat/my-feature

# 2. Add redirect (count "../" hops from .worktrees/feat/.beads/ to project root)
echo "../../../.beads" > .worktrees/feat/.beads/redirect

# 3. Verify
cd .worktrees/feat && br where && cd -

# 4. Work normally from the worktree
cd .worktrees/feat
br create "New task" -p 1 -t task
br ready --json

# 5. Commit JSONL from the main repo
cd /code/myproject
br sync --flush-only
git add .beads/ && git commit -m "chore: update beads JSONL"
```

## Troubleshooting

```bash
br doctor                    # Full diagnostics
br dep cycles                # Must be empty
br config --list             # Check settings
```

**Worktree error** (`'main' is already checked out`):
```bash
git branch beads-sync main
br config set sync.branch beads-sync
```

**Redirect target not found**: The path in `.beads/redirect` does not resolve
to an existing directory. Check that relative paths are counted from the
`.beads/` directory, not the project root.

**Redirect loop detected**: Two or more redirect files point at each other.
Run `br where` from each location to trace the chain, then fix the cycle.

**Redirect chain exceeds max depth**: More than 10 redirect hops. Simplify
the chain -- most setups need exactly one hop.

---

## References

| Topic | File |
|-------|------|
| Full command reference | [COMMANDS.md](references/COMMANDS.md) |
| Configuration details | [CONFIG.md](references/CONFIG.md) |
| Troubleshooting guide | [TROUBLESHOOTING.md](references/TROUBLESHOOTING.md) |
| Multi-agent patterns | [INTEGRATION.md](references/INTEGRATION.md) |
