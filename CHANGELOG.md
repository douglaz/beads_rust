# Changelog

Agent-facing changelog for the full history of `br` (beads\_rust).

Scope: project inception on 2026-01-15 through the latest commits past `v0.1.30` on 2026-03-21. Covers 933 commits across 31 tagged versions.

Organized by **landed capabilities** first, then by **per-version detail**. Each section includes live commit and release/tag links so an agent can jump straight to the implementation.

---

## How to Read This Document

1. **Capability sections (1--9)** explain *what* was built and *why*, with representative commit links.
2. **Per-version notes** below them give the chronological release-by-release breakdown.
3. `Kind` in the version timeline distinguishes a published **GitHub Release** (with binaries) from a bare **git tag** used for fast stabilization cuts.
4. Commit links: `https://github.com/Dicklesworthstone/beads_rust/commit/<HASH>`
5. Release links: `https://github.com/Dicklesworthstone/beads_rust/releases/tag/<TAG>`
6. Tag-only links: `https://github.com/Dicklesworthstone/beads_rust/tree/<TAG>`

---

## Unreleased (after v0.1.30)

Changes landed on `main` after the `v0.1.30` tag (2026-03-20):

- **Atomic config writes**: PID-scoped temp files prevent partial-write corruption ([`e3a00e3`](https://github.com/Dicklesworthstone/beads_rust/commit/e3a00e3)).
- **Graceful missing-dependency fallback**: storage and graph paths no longer crash on dangling dep references ([`617572f`](https://github.com/Dicklesworthstone/beads_rust/commit/617572f)).
- **Blocked-cache hardening**: single-row inserts, deferred invalidation, INSERT OR REPLACE, graceful read fallbacks ([`ad27f47`](https://github.com/Dicklesworthstone/beads_rust/commit/ad27f47), [`acedf9d`](https://github.com/Dicklesworthstone/beads_rust/commit/acedf9d), [`f687166`](https://github.com/Dicklesworthstone/beads_rust/commit/f687166)).
- **Concurrent-write safety**: auto-import SyncConflict downgraded to warning ([`4bc6681`](https://github.com/Dicklesworthstone/beads_rust/commit/4bc6681)).
- **Doctor improvements**: warn when root `.gitignore` hides `.beads/.gitignore` ([`5f1da48`](https://github.com/Dicklesworthstone/beads_rust/commit/5f1da48)).
- **Lazy config loading** and reduced sync lock contention ([`a690d58`](https://github.com/Dicklesworthstone/beads_rust/commit/a690d58)).
- **Centralized ID resolution** into `resolve_issue_id(s)` helpers ([`94c9138`](https://github.com/Dicklesworthstone/beads_rust/commit/94c9138)).
- **Concurrency stress test**: close/update/reopen blocked-cache integrity ([`30d95b4`](https://github.com/Dicklesworthstone/beads_rust/commit/30d95b4)).
- **CI**: renamed release body file to `RELEASE_NOTES.md` ([`9689bd2`](https://github.com/Dicklesworthstone/beads_rust/commit/9689bd2)).

---

## Version Timeline

| Version | Kind | Date | One-line Summary |
|---------|------|------|------------------|
| [`v0.1.30`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.30) | Release | 2026-03-21 | Stats/list/lint/count/stale expansion, deferred blocked-cache, pagination fixes |
| [`v0.1.29`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.29) | Release | 2026-03-19 | frankensqlite v0.1.1 upgrade (~100x write perf), MCP server, CSV injection mitigation |
| [`v0.1.28`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.28) | Release | 2026-03-14 | Database-family snapshots, quarantine model, workspace failure corpus |
| [`v0.1.27`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.27) | Release | 2026-03-13 | Workspace health contract, concurrency coverage, TOON/routing/quiet completion |
| [`v0.1.26`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.26) | Release | 2026-03-11 | Cross-project routing for all mutations, TOON/quiet mode completion |
| [`v0.1.25`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.25) | Release | 2026-03-11 | Sync safety (SyncConflict, 3-way merge), output modes, CLI/storage refactors |
| [`v0.1.24`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.24) | Release | 2026-03-08 | WAL journal support, SQL-aware schema splitter, dep tree visualization |
| [`v0.1.23`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.23) | Release | 2026-03-07 | --db override, graceful config fallback, installer fix |
| [`v0.1.22`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.22) | Release | 2026-03-07 | Doctor --repair, auto DB recovery from JSONL, self-update/install hardening |
| [`v0.1.21`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.21) | Release | 2026-03-04 | Parallel write data loss fix, Claude Code skill, Rust 2024 edition adoption |
| [`v0.1.20`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.20) | Release | 2026-02-26 | fsqlite macOS VFS fix, Draft status, community bug fixes |
| [`v0.1.19`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.19) | Release | 2026-02-23 | CI release-build fix (partial release, ARM64 workaround) |
| [`v0.1.18`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.18) | Tag | 2026-02-23 | Switch Linux builds from musl to gnu |
| [`v0.1.17`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.17) | Tag | 2026-02-23 | CI target installation fix |
| [`v0.1.16`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.16) | Tag | 2026-02-23 | Version bump for release pipeline |
| [`v0.1.15`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.15) | Tag | 2026-02-23 | frankensqlite migration, FTS5 search, self-update, license change |
| [`v0.1.14`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.14) | Release | 2026-02-15 | Atomic claim guard, sync preflight, NothingToDo exit code, schema migration speedup |
| [`v0.1.13`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.13) | Release | 2026-02-01 | Shell completions, rename-prefix sync, rich output across all commands |
| [`v0.1.12`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.12) | Release | 2026-01-29 | LIKE escaping fix, JSON snapshot tests, output mode consistency |
| [`v0.1.11`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.11) | Tag | 2026-01-28 | --wrap flag for blocked, structured error validation tests |
| [`v0.1.10`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.10) | Tag | 2026-01-28 | TOON output format, Nix flake, schema command, VCS integration guide |
| [`v0.1.9`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.9) | Tag | 2026-01-23 | ID-prefix dot validation, ready-query SQL fix, CLI output improvements |
| [`v0.1.8`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.8) | Tag | 2026-01-22 | Rich output foundation, conformance harness, storage schema expansion |
| [`v0.1.7`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.7) | Release | 2026-01-18 | Bulletproof installer, AGENTS.md management, macOS CI fix |
| [`v0.1.6`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.6) | Tag | 2026-01-18 | Import order fix for cargo fmt |
| [`v0.1.5`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.5) | Tag | 2026-01-18 | Conformance test skip for all files |
| [`v0.1.4`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.4) | Tag | 2026-01-18 | Conformance test skip |
| [`v0.1.3`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.3) | Tag | 2026-01-18 | Benchmark test bd-skip check |
| [`v0.1.2`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.2) | Tag | 2026-01-18 | Benchmark CI skip |
| [`v0.1.1`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.1) | Tag | 2026-01-18 | Post-launch cleanup (gitignore, build artifacts) |
| [`v0.1.0`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.0) | Draft Release | 2026-01-18 | Initial public release |

---

## Capability Sections

### 1) The Classic Beads Port: A Real Rust CLI

`br` began as a deliberate freeze of the classic SQLite + JSONL architecture from Steve Yegge's [beads](https://github.com/steveyegge/beads). The initial development wave (2026-01-15 through 2026-01-18) built the Rust crate, core model/storage layers, command scaffold, and non-invasive philosophy that still defines the project.

**What shipped:**

- Rust crate with nightly toolchain, core model types, validation, ID generation, content hashing, and logging.
- SQLite primary storage with JSONL import/export as the collaboration surface.
- Full command-surface parity: `init`, `create`, `list`, `show`, `update`, `close`, `reopen`, `ready`, `blocked`, `search`, `dep`, `comments`, `config`, `doctor`, `sync`, `history`, and related workflow commands.
- Cross-platform binaries: Linux (x86\_64, aarch64), macOS (x86\_64, Apple Silicon), Windows.
- Explicit documentation of classic-only scope and non-invasive rules (br never runs git commands automatically).

**Key commits:**

- [`38cd152`](https://github.com/Dicklesworthstone/beads_rust/commit/38cd152) -- AGENTS.md and original porting plan.
- [`ec14cba`](https://github.com/Dicklesworthstone/beads_rust/commit/ec14cba) -- initialized Rust project and codified legacy behavior expectations.
- [`562e021`](https://github.com/Dicklesworthstone/beads_rust/commit/562e021) -- core model types.
- [`16c98b8`](https://github.com/Dicklesworthstone/beads_rust/commit/16c98b8) -- CLI command scaffold.
- [`229ec5a`](https://github.com/Dicklesworthstone/beads_rust/commit/229ec5a) -- doctor diagnostics and sync CLI modules.

---

### 2) Sync Safety as a First-Class System

The sync engine evolved from basic JSONL import/export into a guarded subsystem with path allowlisting, 3-way merge logic, SyncConflict handling, local history backups, and crash-friendly snapshot/quarantine infrastructure. This is central to the "non-invasive" promise.

**What shipped:**

- Path canonicalization and allowlisting for all sync I/O.
- "No git operations" guarantee enforced in code and tested in regression suites.
- External JSONL opt-in instead of silent broad filesystem writes.
- 3-way merge infrastructure with `sync_equals()` comparison ([`caace45`](https://github.com/Dicklesworthstone/beads_rust/commit/caace45)).
- `SyncConflict` error type to prevent silent data loss on auto-import ([`1017b00`](https://github.com/Dicklesworthstone/beads_rust/commit/1017b00)).
- Local `.br_history` backups and failure-aware export/import.
- Database-family snapshotting and sidecar quarantine ([`e430d4c`](https://github.com/Dicklesworthstone/beads_rust/commit/e430d4c)).
- Re-read JSONL before flush in `--no-db` mode to avoid clobbering concurrent writes ([`968d2e0`](https://github.com/Dicklesworthstone/beads_rust/commit/968d2e0)).
- Deterministic export ordering and streaming git log ([`6e7ea09`](https://github.com/Dicklesworthstone/beads_rust/commit/6e7ea09)).

---

### 3) Testing, Conformance, Benchmarks, and Release Automation

`br` invested heavily in testing infrastructure from day one. The project has unit tests, E2E harnesses, `bd`-vs-`br` conformance suites, snapshot/golden tests, synthetic benchmarks, and full CI/release/installer automation.

**What shipped:**

- Structured E2E test harnesses with logging and artifact capture.
- `bd` vs `br` conformance harnesses to keep classic behavior grounded.
- Snapshot/golden testing for output surfaces.
- Synthetic and real-dataset benchmark suites with regression detection scripts.
- CI workflows: build matrix, clippy, fmt, audit, caching.
- Release workflows: preflight checks, cross-platform builds, signing/checksums, installer automation.
- Curl-pipe-bash installer with fallback to source build ([`f09877d`](https://github.com/Dicklesworthstone/beads_rust/commit/f09877d)).
- Workspace-failure fixtures and replay harnesses ([`046c311`](https://github.com/Dicklesworthstone/beads_rust/commit/046c311), [`05dc2ec`](https://github.com/Dicklesworthstone/beads_rust/commit/05dc2ec)).
- Concurrency stress tests for close/update/reopen mixes ([`30d95b4`](https://github.com/Dicklesworthstone/beads_rust/commit/30d95b4), [`66ee59e`](https://github.com/Dicklesworthstone/beads_rust/commit/66ee59e)).
- Dataset registry with git commit detection ([`1910db4`](https://github.com/Dicklesworthstone/beads_rust/commit/1910db4)).

---

### 4) Rich Output and Agent-First Output Modes

`br` serves both humans and automation. The project grew from plain text to support rich terminal rendering (via `rich_rust`), structured JSON, TOON (Token-Optimized Object Notation), quiet mode, and a consistent `OutputContext` that propagates through the command dispatcher.

**What shipped:**

- Rich terminal output: panels, tables, tree connectors, colored status indicators.
- Stable JSON output for all major commands (`--json` or `--robot` flags).
- TOON output across: show, list, create, close, update, ready, blocked, stats, count, epic, stale, history, orphans, query, graph, audit, lint, version, dep, search ([`6a1618c`](https://github.com/Dicklesworthstone/beads_rust/commit/6a1618c), [`9565af0`](https://github.com/Dicklesworthstone/beads_rust/commit/9565af0), [`02c3bde`](https://github.com/Dicklesworthstone/beads_rust/commit/02c3bde)).
- Quiet mode that suppresses human chatter without breaking robot surfaces ([`9b43240`](https://github.com/Dicklesworthstone/beads_rust/commit/9b43240)).
- Long/pretty output modes with box-drawing tree connectors ([`a81fa2b`](https://github.com/Dicklesworthstone/beads_rust/commit/a81fa2b)).
- Graceful serialization (no panics) on JSON/TOON errors ([`5642445`](https://github.com/Dicklesworthstone/beads_rust/commit/5642445)).

---

### 5) Routing, Agent Coordination, and MCP

`br` participates in larger agentic workflows. Cross-project routing, external dependency syntax, the MCP server, agent-focused commands, and centralized ID resolution make it a coordination tool, not just a local TODO list.

**What shipped:**

- Cross-project issue routing with batched dispatch ([`be49fef`](https://github.com/Dicklesworthstone/beads_rust/commit/be49fef)), extended to all mutation commands ([`9b43240`](https://github.com/Dicklesworthstone/beads_rust/commit/9b43240)).
- External dependency references (`external:<project>:<capability>`).
- Optional MCP server for AI agent integration ([`2195144`](https://github.com/Dicklesworthstone/beads_rust/commit/2195144)), with hardened tools and prompt quality ([`7a1c17a`](https://github.com/Dicklesworthstone/beads_rust/commit/7a1c17a), [`8f35a53`](https://github.com/Dicklesworthstone/beads_rust/commit/8f35a53)).
- Atomic claim guard with IMMEDIATE transaction for safe multi-agent work claiming ([`0a52ac7`](https://github.com/Dicklesworthstone/beads_rust/commit/0a52ac7)).
- Official Claude Code skill for `br` ([`578d02f`](https://github.com/Dicklesworthstone/beads_rust/commit/578d02f)).
- Centralized `resolve_issue_id(s)` helpers ([`94c9138`](https://github.com/Dicklesworthstone/beads_rust/commit/94c9138)).

---

### 6) Workspace Reliability and Failure Modeling

The repo moved from "green tests" toward explicit failure taxonomies, workspace-health contracts, incident bundles, failure corpora, evolution scenarios, replay harnesses, and stronger doctor/recovery behavior.

**What shipped:**

- Documented workspace-health contract and invariant matrix ([`5cfc4e0`](https://github.com/Dicklesworthstone/beads_rust/commit/5cfc4e0)).
- Sanitized fixture corpus for corrupted/drifted workspaces ([`05dc2ec`](https://github.com/Dicklesworthstone/beads_rust/commit/05dc2ec)).
- Database-family snapshot infrastructure and sidecar quarantine ([`e430d4c`](https://github.com/Dicklesworthstone/beads_rust/commit/e430d4c)).
- Doctor `--repair` flag to rebuild DB from JSONL ([`3150f9e`](https://github.com/Dicklesworthstone/beads_rust/commit/3150f9e)).
- Automatic database recovery during mutation commands ([`21a1031`](https://github.com/Dicklesworthstone/beads_rust/commit/21a1031)).
- JSONL recovery generalized to all mutation commands ([`1e163ed`](https://github.com/Dicklesworthstone/beads_rust/commit/1e163ed)).
- Probe helper to distinguish corruption from application errors ([`ca701a7`](https://github.com/Dicklesworthstone/beads_rust/commit/ca701a7)).

---

### 7) Multi-Agent Concurrency and Blocked-Cache Correctness

A sustained hardening track making `br` behave correctly when dozens of agents write simultaneously. The blocked-cache, auto-import, and read/write contention paths received repeated attention.

**What shipped:**

- Deferred blocked-cache refresh with stale-marker protocol ([`674b9bd`](https://github.com/Dicklesworthstone/beads_rust/commit/674b9bd), [`45232f6`](https://github.com/Dicklesworthstone/beads_rust/commit/45232f6)).
- Atomic DELETE+INSERT rewrite of blocked cache ([`0a9609e`](https://github.com/Dicklesworthstone/beads_rust/commit/0a9609e)).
- Batched status mutations with stale-marking fallback ([`afa8d06`](https://github.com/Dicklesworthstone/beads_rust/commit/afa8d06)).
- Graceful fallback on all blocked-cache reads ([`acedf9d`](https://github.com/Dicklesworthstone/beads_rust/commit/acedf9d)).
- Convergence-based blocked-cache propagation replacing silent depth cap ([`d5f124c`](https://github.com/Dicklesworthstone/beads_rust/commit/d5f124c)).
- Fix for parallel write data loss from dead `busy_timeout` ([`f83a9b0`](https://github.com/Dicklesworthstone/beads_rust/commit/f83a9b0)).
- `in_progress` issues excluded from `ready` output ([`2a409df`](https://github.com/Dicklesworthstone/beads_rust/commit/2a409df), [`f226f66`](https://github.com/Dicklesworthstone/beads_rust/commit/f226f66)).
- Concurrent auto-import conflicts downgraded to warnings where safe ([`4bc6681`](https://github.com/Dicklesworthstone/beads_rust/commit/4bc6681)).

---

### 8) Performance and Storage Throughput

Performance work spans the full history but became a headline with the frankensqlite migration and the v0.1.1 upgrade that delivered roughly 100x write throughput improvement.

**What shipped:**

- Migration from `rusqlite` to `frankensqlite` (pure-Rust SQLite), started in v0.1.15 ([`d3d9bce`](https://github.com/Dicklesworthstone/beads_rust/commit/d3d9bce)).
- frankensqlite v0.1.1 upgrade for ~100x write perf ([`39f3e0e`](https://github.com/Dicklesworthstone/beads_rust/commit/39f3e0e)).
- Write contention eliminated from read-only CLI commands ([`33335b3`](https://github.com/Dicklesworthstone/beads_rust/commit/33335b3)).
- Blocked-by computation moved to Rust, reducing allocations ([`8a5522f`](https://github.com/Dicklesworthstone/beads_rust/commit/8a5522f)).
- Streaming JSONL serialization with reusable buffer ([`8d3c9bf`](https://github.com/Dicklesworthstone/beads_rust/commit/8d3c9bf)).
- SQL-level label filtering and fast-path SQL limit push-down ([`0b88b36`](https://github.com/Dicklesworthstone/beads_rust/commit/0b88b36), [`9d3473d`](https://github.com/Dicklesworthstone/beads_rust/commit/9d3473d)).
- Lazy config loading and reduced sync-lock contention ([`a690d58`](https://github.com/Dicklesworthstone/beads_rust/commit/a690d58)).
- Zero-allocation JSON length measurement for TOON stats ([`db036bc`](https://github.com/Dicklesworthstone/beads_rust/commit/db036bc)).
- Recursive CTE replaced with Rust-side BFS for descendant queries ([`58597df`](https://github.com/Dicklesworthstone/beads_rust/commit/58597df)).

---

### 9) Security and Ecosystem Hardening

Security, licensing, and ecosystem integration improvements that accumulated across releases.

**What shipped:**

- CSV formula injection mitigation ([`ab5356d`](https://github.com/Dicklesworthstone/beads_rust/commit/ab5356d)).
- License changed to MIT + OpenAI/Anthropic Rider ([`b91c42b`](https://github.com/Dicklesworthstone/beads_rust/commit/b91c42b)).
- Default issue prefix changed from `bd` to `br` ([`e6e7dcb`](https://github.com/Dicklesworthstone/beads_rust/commit/e6e7dcb)).
- Nix flake support ([`d5e9821`](https://github.com/Dicklesworthstone/beads_rust/commit/d5e9821)).
- Shell completions for all CLI arguments ([`603c53b`](https://github.com/Dicklesworthstone/beads_rust/commit/603c53b), [`4c2f107`](https://github.com/Dicklesworthstone/beads_rust/commit/4c2f107)).
- Symlink/gitdir invariant bypass prevention ([`3a878c2`](https://github.com/Dicklesworthstone/beads_rust/commit/3a878c2)).
- Table/column whitelist for `has_missing_issue_reference` ([`014e676`](https://github.com/Dicklesworthstone/beads_rust/commit/014e676)).

---

## Per-Version Detail

### [`v0.1.30`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.30) -- 2026-03-21

**Theme:** Command-surface expansion and concurrency follow-through.

- **Stats expansion**: major new aggregate metrics in stats command ([`ac4ff74`](https://github.com/Dicklesworthstone/beads_rust/commit/ac4ff74), [`4703dff`](https://github.com/Dicklesworthstone/beads_rust/commit/4703dff), [`b634768`](https://github.com/Dicklesworthstone/beads_rust/commit/b634768)).
- **List command**: new output modes, paginated JSON envelope, offset-after-filtering fix ([`e273d58`](https://github.com/Dicklesworthstone/beads_rust/commit/e273d58), [`36a5ff8`](https://github.com/Dicklesworthstone/beads_rust/commit/36a5ff8)).
- **Lint/count/stale/blocked/epic**: expanded output, additional metrics, E2E test suites ([`c4f861c`](https://github.com/Dicklesworthstone/beads_rust/commit/c4f861c), [`3126725`](https://github.com/Dicklesworthstone/beads_rust/commit/3126725), [`0987d6e`](https://github.com/Dicklesworthstone/beads_rust/commit/0987d6e), [`0333b98`](https://github.com/Dicklesworthstone/beads_rust/commit/0333b98)).
- **Deferred blocked-cache refresh** for dependency mutations to reduce DB lock contention ([`45232f6`](https://github.com/Dicklesworthstone/beads_rust/commit/45232f6)).
- **Doctor**: resolve concurrent DB corruption false positives ([`3a1feef`](https://github.com/Dicklesworthstone/beads_rust/commit/3a1feef)).
- **Batched mutations**: stale-cache pre-marking with routing test coverage ([`cdd9cb4`](https://github.com/Dicklesworthstone/beads_rust/commit/cdd9cb4)).
- **Docs**: implement PRs #73, #163, #166 -- body alias, RUST\_LOG=error docs, broken link fix ([`144070e`](https://github.com/Dicklesworthstone/beads_rust/commit/144070e)).
- **Mixed prefix support**: defer prefix enforcement to explicit `--rename-prefix` ([`d012e19`](https://github.com/Dicklesworthstone/beads_rust/commit/d012e19)).

34 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.29...v0.1.30).

---

### [`v0.1.29`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.29) -- 2026-03-19

**Theme:** frankensqlite write-performance leap, MCP server, and security hardening.

- **frankensqlite v0.1.1 upgrade**: ~100x write performance improvement ([`39f3e0e`](https://github.com/Dicklesworthstone/beads_rust/commit/39f3e0e)).
- **MCP server**: optional Model Context Protocol server for AI agent integration ([`2195144`](https://github.com/Dicklesworthstone/beads_rust/commit/2195144)), with error consolidation and prompt quality improvements ([`7a1c17a`](https://github.com/Dicklesworthstone/beads_rust/commit/7a1c17a), [`8f35a53`](https://github.com/Dicklesworthstone/beads_rust/commit/8f35a53)).
- **CSV formula injection mitigation** ([`ab5356d`](https://github.com/Dicklesworthstone/beads_rust/commit/ab5356d)).
- **Default prefix**: changed from `bd` to `br`, plus delete `--hard` JSONL purge ([`e6e7dcb`](https://github.com/Dicklesworthstone/beads_rust/commit/e6e7dcb)).
- **TOON output for graph** command ([`02c3bde`](https://github.com/Dicklesworthstone/beads_rust/commit/02c3bde)).
- **Unicode-width-aware truncation** in dep tree ([`72b8560`](https://github.com/Dicklesworthstone/beads_rust/commit/72b8560)).
- **Atomic config writes** and empty-comment validation ([`1796519`](https://github.com/Dicklesworthstone/beads_rust/commit/1796519)).
- **Ready**: exclude in\_progress issues from ready work queue ([`f226f66`](https://github.com/Dicklesworthstone/beads_rust/commit/f226f66)).
- **Delete**: show full transitive cascade closure in dry-run preview ([`94c3486`](https://github.com/Dicklesworthstone/beads_rust/commit/94c3486)).
- **Orphans**: early return in Quiet mode, OrphanRenderMode enum ([`0fe3ae8`](https://github.com/Dicklesworthstone/beads_rust/commit/0fe3ae8), [`f00a2be`](https://github.com/Dicklesworthstone/beads_rust/commit/f00a2be)).
- **fsqlite compatibility**: harden schema and query paths ([`47fa201`](https://github.com/Dicklesworthstone/beads_rust/commit/47fa201)).

43 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.28...v0.1.29).

---

### [`v0.1.28`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.28) -- 2026-03-14

**Theme:** Narrow stabilization cut.

- Cleaned up stale `.rebuild-failed` recovery artifacts from test fixtures ([`cd546f9`](https://github.com/Dicklesworthstone/beads_rust/commit/cd546f9)).

2 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.27...v0.1.28).

---

### [`v0.1.27`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.27) -- 2026-03-13

**Theme:** Reliability, workspace health, concurrency coverage, and output architecture.

- **Database-family snapshot infrastructure**, quarantine support, and external JSONL safety model ([`e430d4c`](https://github.com/Dicklesworthstone/beads_rust/commit/e430d4c)).
- **Workspace failure replay tests** and evolution plan framework ([`046c311`](https://github.com/Dicklesworthstone/beads_rust/commit/046c311)).
- **Workspace health contract** documentation and diagnostic hardening ([`5cfc4e0`](https://github.com/Dicklesworthstone/beads_rust/commit/5cfc4e0)).
- **Deferred blocked-cache refresh** with stale-marker protocol ([`674b9bd`](https://github.com/Dicklesworthstone/beads_rust/commit/674b9bd)).
- **Automatic database recovery** during mutation commands ([`21a1031`](https://github.com/Dicklesworthstone/beads_rust/commit/21a1031)).
- **JSONL recovery** generalized to all mutation commands ([`1e163ed`](https://github.com/Dicklesworthstone/beads_rust/commit/1e163ed)).
- **Incremental blocked-cache updates** and bulk cycle-check adjacency loading ([`d3d3e64`](https://github.com/Dicklesworthstone/beads_rust/commit/d3d3e64)).
- **TOON output** added to audit, lint, version, count, epic, stale, history, orphans, query ([`9565af0`](https://github.com/Dicklesworthstone/beads_rust/commit/9565af0), [`6a1618c`](https://github.com/Dicklesworthstone/beads_rust/commit/6a1618c)).
- **Cross-project routing** extended to all mutation commands, quiet mode completion ([`9b43240`](https://github.com/Dicklesworthstone/beads_rust/commit/9b43240)).
- **Blocked cache rewritten** as atomic DELETE+INSERT with ForeignKeyGuard RAII ([`0a9609e`](https://github.com/Dicklesworthstone/beads_rust/commit/0a9609e)).
- **Cycle detection** switched to lazy per-node BFS ([`f2e20d4`](https://github.com/Dicklesworthstone/beads_rust/commit/f2e20d4)).
- **Symlink/gitdir bypass prevention** via early canonicalization ([`3a878c2`](https://github.com/Dicklesworthstone/beads_rust/commit/3a878c2)).
- **Concurrency E2E**: interleaved command families, routed workspace contention ([`66ee59e`](https://github.com/Dicklesworthstone/beads_rust/commit/66ee59e)).
- **Markdown import**: `--parent` and `--dry-run` support ([`c1b8541`](https://github.com/Dicklesworthstone/beads_rust/commit/c1b8541)).

48 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.26...v0.1.27).

---

### [`v0.1.26`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.26) -- 2026-03-11

**Theme:** Routing and output mode completion.

- **Cross-project issue routing** with batched dispatch ([`be49fef`](https://github.com/Dicklesworthstone/beads_rust/commit/be49fef)).
- **Show/blocked/ready/stats** enhanced with routing support and richer output ([`7391be3`](https://github.com/Dicklesworthstone/beads_rust/commit/7391be3)).
- **No-db flush safety**: re-read JSONL before flush to prevent clobbering concurrent writes ([`968d2e0`](https://github.com/Dicklesworthstone/beads_rust/commit/968d2e0)).
- Minor cleanups across close, comments, defer, delete, dep, epic, label, q, reopen ([`dc62fff`](https://github.com/Dicklesworthstone/beads_rust/commit/dc62fff)).

7 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.25...v0.1.26).

---

### [`v0.1.25`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.25) -- 2026-03-11

**Theme:** Sync safety maturity, output refinement, and deep CLI/storage refactoring.

- **SyncConflict error** to prevent silent data loss on auto-import ([`1017b00`](https://github.com/Dicklesworthstone/beads_rust/commit/1017b00)).
- **sync\_equals()** implementation for 3-way merge ([`caace45`](https://github.com/Dicklesworthstone/beads_rust/commit/caace45)).
- **Long/pretty output modes** with box-drawing tree connectors ([`a81fa2b`](https://github.com/Dicklesworthstone/beads_rust/commit/a81fa2b)).
- **Bidirectional dep traversal** and improved cycle detection ([`004bab8`](https://github.com/Dicklesworthstone/beads_rust/commit/004bab8)).
- **Today/yesterday time keywords** and DST-safe helpers ([`7e0d26d`](https://github.com/Dicklesworthstone/beads_rust/commit/7e0d26d)).
- **External\_ref uniqueness** enforcement and atomic blocked-cache migration ([`fc656d9`](https://github.com/Dicklesworthstone/beads_rust/commit/fc656d9)).
- **Write contention eliminated** from read-only CLI commands ([`33335b3`](https://github.com/Dicklesworthstone/beads_rust/commit/33335b3)).
- **Blocked-by computation** moved to Rust, reducing allocations ([`8a5522f`](https://github.com/Dicklesworthstone/beads_rust/commit/8a5522f)).
- **Phased startup lifecycle**, child counters, ID collision retry ([`eb3d0c0`](https://github.com/Dicklesworthstone/beads_rust/commit/eb3d0c0)).
- **Schema v3 migration** for NOT NULL filter columns ([`092fdc2`](https://github.com/Dicklesworthstone/beads_rust/commit/092fdc2)).
- **Pipe-safety wrap** for curl|bash truncation edge case ([`bb24002`](https://github.com/Dicklesworthstone/beads_rust/commit/bb24002)).

57 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.24...v0.1.25).

---

### [`v0.1.24`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.24) -- 2026-03-08

**Theme:** WAL concurrency and multi-agent safety.

- **SQLite WAL journal support**, git context fixes, atomic ops hardening ([`02a75ec`](https://github.com/Dicklesworthstone/beads_rust/commit/02a75ec)).
- **SQL-aware statement splitter** replacing naive `split(';')` ([`45015bc`](https://github.com/Dicklesworthstone/beads_rust/commit/45015bc)).
- **Convergence-based blocked-cache propagation** replacing silent depth cap ([`d5f124c`](https://github.com/Dicklesworthstone/beads_rust/commit/d5f124c)).
- **InheritedOutputMode** for consistent format propagation across subcommands ([`b1b9d67`](https://github.com/Dicklesworthstone/beads_rust/commit/b1b9d67)).
- **Dependency tree visualization** enhancements, theming, quiet mode ([`e30be1e`](https://github.com/Dicklesworthstone/beads_rust/commit/e30be1e)).
- **crates.io packaging**: exclude list and readme field ([`32c0fb2`](https://github.com/Dicklesworthstone/beads_rust/commit/32c0fb2)).

8 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.23...v0.1.24).

---

### [`v0.1.23`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.23) -- 2026-03-07

**Theme:** Config stabilization and installer fix.

- **`--db` override** respected across all subcommands with graceful fallback ([`b91ee46`](https://github.com/Dicklesworthstone/beads_rust/commit/b91ee46)).
- **Enhanced diff output**, CLI help styling, config validation ([`f81055a`](https://github.com/Dicklesworthstone/beads_rust/commit/f81055a)).
- **Installer**: remove non-functional musl binary attempt on Linux x86\_64 ([`0c9f1de`](https://github.com/Dicklesworthstone/beads_rust/commit/0c9f1de)).

4 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.22...v0.1.23).

---

### [`v0.1.22`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.22) -- 2026-03-07

**Theme:** Doctor repair, auto-recovery, and compatibility hardening.

- **Doctor `--repair`**: rebuild entire DB from JSONL export ([`3150f9e`](https://github.com/Dicklesworthstone/beads_rust/commit/3150f9e)).
- **Automatic SQLite database recovery** from JSONL export ([`4d35e55`](https://github.com/Dicklesworthstone/beads_rust/commit/4d35e55)).
- **Runtime-compatible schema repair** and table rebuild safety ([`23ef6bf`](https://github.com/Dicklesworthstone/beads_rust/commit/23ef6bf)).
- **Config prefix inference** from JSONL to prevent bd-\* fallback ([`382832d`](https://github.com/Dicklesworthstone/beads_rust/commit/382832d)).
- **Self-update**: ensure archive extraction works in release builds ([`a555c9e`](https://github.com/Dicklesworthstone/beads_rust/commit/a555c9e)).
- **musl static build** for Linux portability ([`15ca9c9`](https://github.com/Dicklesworthstone/beads_rust/commit/15ca9c9)).
- **Transactional imports** and comprehensive error propagation ([`f93df50`](https://github.com/Dicklesworthstone/beads_rust/commit/f93df50)).
- **Label dedup/rename** hardening, comment parsing safety ([`887e6f7`](https://github.com/Dicklesworthstone/beads_rust/commit/887e6f7)).
- **Reopened event** recorded when transitioning from terminal to non-terminal status ([`30ee737`](https://github.com/Dicklesworthstone/beads_rust/commit/30ee737)).
- **`br q`**: added `-d`, `--parent`, `-e` flags and truncation warning ([`fe18252`](https://github.com/Dicklesworthstone/beads_rust/commit/fe18252)).
- **Installer**: Windows/zip support, refactored binary discovery ([`bbf674f`](https://github.com/Dicklesworthstone/beads_rust/commit/bbf674f)).

30 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.21...v0.1.22).

---

### [`v0.1.21`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.21) -- 2026-03-04

**Theme:** Concurrency safety, Rust 2024 edition, and Claude Code integration.

- **Parallel write data loss fix**: dead `busy_timeout` was causing silent data loss ([`f83a9b0`](https://github.com/Dicklesworthstone/beads_rust/commit/f83a9b0)).
- **Claude Code skill**: official skill for `br` integration ([`578d02f`](https://github.com/Dicklesworthstone/beads_rust/commit/578d02f)).
- **Rust 2024 edition**: adopted let-chains and idiomatic clippy patterns ([`070d149`](https://github.com/Dicklesworthstone/beads_rust/commit/070d149)).
- **Blocked cache**: refresh after dep changes, cycle detection fix, atomicity improvement ([`84e71cd`](https://github.com/Dicklesworthstone/beads_rust/commit/84e71cd)).
- **Schema**: remove PRIMARY KEY from config/metadata tables, clean up migrations ([`648d46b`](https://github.com/Dicklesworthstone/beads_rust/commit/648d46b)).
- **frankensqlite**: multiple upstream fixes for B-tree cursor, page-count header, page-header ([`0e4b5df`](https://github.com/Dicklesworthstone/beads_rust/commit/0e4b5df), [`23e9797`](https://github.com/Dicklesworthstone/beads_rust/commit/23e9797)).
- **Schema migration**: repair 4 bugs in `rebuild_issues_table` ([`3a4faf2`](https://github.com/Dicklesworthstone/beads_rust/commit/3a4faf2)).
- **5 bug fixes** (issues #104--#108) ([`c6529f4`](https://github.com/Dicklesworthstone/beads_rust/commit/c6529f4)).
- **Auto-flush/auto-import** flags resolved from merged config layers ([`d4586cb`](https://github.com/Dicklesworthstone/beads_rust/commit/d4586cb)).

16 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.20...v0.1.21).

---

### [`v0.1.20`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.20) -- 2026-02-26

**Theme:** macOS VFS compatibility, Draft status, and community bug fixes.

- **fsqlite macOS fixes**: c\_short VFS lock and type mismatch fixes ([`cd5bc27`](https://github.com/Dicklesworthstone/beads_rust/commit/cd5bc27), [`6a7678c`](https://github.com/Dicklesworthstone/beads_rust/commit/6a7678c)).
- **Draft status variant** for pre-execution issues ([`82560a5`](https://github.com/Dicklesworthstone/beads_rust/commit/82560a5)).
- **6 community-reported issues** resolved (#85, #86, #87, #88, #91, #92) ([`75dd6f1`](https://github.com/Dicklesworthstone/beads_rust/commit/75dd6f1)).
- **CI**: switch to gnu targets for pure-Rust UnixVfs ([`4adeb86`](https://github.com/Dicklesworthstone/beads_rust/commit/4adeb86)).
- **linux\_arm64** added to changelog platforms table ([`c7b654b`](https://github.com/Dicklesworthstone/beads_rust/commit/c7b654b)).

11 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.19...v0.1.20).

---

### [`v0.1.19`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.19) -- 2026-02-23

**Theme:** CI release-build fix.

- **Allow partial release** and temporarily disable linux\_arm64 ([`e67031b`](https://github.com/Dicklesworthstone/beads_rust/commit/e67031b)).

1 commit (narrow release fix). [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.18...v0.1.19).

---

### [`v0.1.18`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.18) -- 2026-02-23 (tag only)

- Switch Linux release builds from musl to gnu ([`bec2a3f`](https://github.com/Dicklesworthstone/beads_rust/commit/bec2a3f)).

---

### [`v0.1.17`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.17) -- 2026-02-23 (tag only)

- Bump to v0.1.17, fix CI target installation for all platforms ([`3874361`](https://github.com/Dicklesworthstone/beads_rust/commit/3874361)).

---

### [`v0.1.16`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.16) -- 2026-02-23 (tag only)

- Version bump for release pipeline ([`729edf8`](https://github.com/Dicklesworthstone/beads_rust/commit/729edf8)).

---

### [`v0.1.15`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.15) -- 2026-02-23 (tag only)

**Theme:** The frankensqlite migration, major architectural shift.

- **frankensqlite migration**: replaced `rusqlite` with pure-Rust `frankensqlite` ([`d3d9bce`](https://github.com/Dicklesworthstone/beads_rust/commit/d3d9bce)), including batch upsert, FTS5 search, and migration framework ([`61920c6`](https://github.com/Dicklesworthstone/beads_rust/commit/61920c6)).
- **Recursive CTE replaced** with Rust-side BFS for descendant queries ([`58597df`](https://github.com/Dicklesworthstone/beads_rust/commit/58597df)).
- **IN-clause chunking** to respect SQLite 999-variable limit ([`8087e48`](https://github.com/Dicklesworthstone/beads_rust/commit/8087e48)).
- **License**: changed to MIT + OpenAI/Anthropic Rider ([`b91c42b`](https://github.com/Dicklesworthstone/beads_rust/commit/b91c42b)).
- **Self-update**: GITHUB\_TOKEN support, Rust target triple mapping ([`a0993d5`](https://github.com/Dicklesworthstone/beads_rust/commit/a0993d5), [`b687c5a`](https://github.com/Dicklesworthstone/beads_rust/commit/b687c5a)).
- **Deferred epic children** marked as blocked in ready cache ([`3867e97`](https://github.com/Dicklesworthstone/beads_rust/commit/3867e97)).
- **agents `--dry-run --json`** produces distinct output with dry\_run/would\_action fields ([`312b40d`](https://github.com/Dicklesworthstone/beads_rust/commit/312b40d)).
- **OutputContext refactoring** and atomic label assignment in quick-create ([`03aa2d1`](https://github.com/Dicklesworthstone/beads_rust/commit/03aa2d1)).
- **External dependency parsing** and label-any filter wired up ([`1adec07`](https://github.com/Dicklesworthstone/beads_rust/commit/1adec07)).
- Dependencies switched from local paths to crates.io ([`6c6ade6`](https://github.com/Dicklesworthstone/beads_rust/commit/6c6ade6), [`b483206`](https://github.com/Dicklesworthstone/beads_rust/commit/b483206)).

26 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.14...v0.1.15).

---

### [`v0.1.14`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.14) -- 2026-02-15

**Theme:** Sync safety, agent ergonomics, and claim guard.

- **Atomic claim guard** with IMMEDIATE transaction for multi-agent work claiming ([`0a52ac7`](https://github.com/Dicklesworthstone/beads_rust/commit/0a52ac7)).
- **Sync preflight guardrails** for JSONL import validation ([`e539185`](https://github.com/Dicklesworthstone/beads_rust/commit/e539185)).
- **NothingToDo exit code** and enhanced show fields ([`e727f6c`](https://github.com/Dicklesworthstone/beads_rust/commit/e727f6c)).
- **Schema migration speedup**: skip DDL/migration when schema is current ([`ee23dc2`](https://github.com/Dicklesworthstone/beads_rust/commit/ee23dc2)).
- **Windows path fix**: use `dunce` for canonicalization to strip `\\?\` prefix ([`4cf7717`](https://github.com/Dicklesworthstone/beads_rust/commit/4cf7717)).
- **self\_update feature gates** completed for `--no-default-features` builds ([`3fa391a`](https://github.com/Dicklesworthstone/beads_rust/commit/3fa391a)).
- **Recursive descendant CTE**: use UNION instead of UNION ALL to prevent duplicates ([`1a3976d`](https://github.com/Dicklesworthstone/beads_rust/commit/1a3976d)).

24 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.13...v0.1.14).

---

### [`v0.1.13`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.13) -- 2026-02-01

**Theme:** Shell completions, rich output, and compatibility work.

- **Shell completions** for all CLI arguments (bash, zsh, fish) ([`603c53b`](https://github.com/Dicklesworthstone/beads_rust/commit/603c53b), [`4c2f107`](https://github.com/Dicklesworthstone/beads_rust/commit/4c2f107)).
- **Rename-prefix sync option** and duplicate external-ref clearing ([`70ec1de`](https://github.com/Dicklesworthstone/beads_rust/commit/70ec1de), [`bbffe2c`](https://github.com/Dicklesworthstone/beads_rust/commit/bbffe2c)).
- **Rich output** integrated across all major commands (stats, label, dep, sync, comments, delete, graph, audit, agents) ([`51dbcbf`](https://github.com/Dicklesworthstone/beads_rust/commit/51dbcbf)).
- **Ready**: `--parent` and `--recursive` flags for scoped filtering ([`ab56d79`](https://github.com/Dicklesworthstone/beads_rust/commit/ab56d79)).
- **Conflicting installation detection** ([`bc7341d`](https://github.com/Dicklesworthstone/beads_rust/commit/bc7341d)).
- **Update**: prevent claiming blocked issues ([`e45fa66`](https://github.com/Dicklesworthstone/beads_rust/commit/e45fa66)).
- **JSONL export normalization** for consistent round-trip hashing ([`b5e83fd`](https://github.com/Dicklesworthstone/beads_rust/commit/b5e83fd)).
- **CI**: musl for Linux builds, ARM64 minisign binary fix ([`7217ae0`](https://github.com/Dicklesworthstone/beads_rust/commit/7217ae0), [`f0c72b5`](https://github.com/Dicklesworthstone/beads_rust/commit/f0c72b5)).
- **Panics replaced** with safe fallbacks ([`b5a687b`](https://github.com/Dicklesworthstone/beads_rust/commit/b5a687b)).

32 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.12...v0.1.13).

---

### [`v0.1.12`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.12) -- 2026-01-29

**Theme:** Output parity and test depth.

- **LIKE pattern escaping** in search queries ([`81266c8`](https://github.com/Dicklesworthstone/beads_rust/commit/81266c8)).
- **Comprehensive JSON output snapshot tests** ([`dcaf4e0`](https://github.com/Dicklesworthstone/beads_rust/commit/dcaf4e0)).
- **E2E output mode consistency tests** ([`4e564ac`](https://github.com/Dicklesworthstone/beads_rust/commit/4e564ac)).
- **CSV escaping** and saved query override tests ([`4933e1b`](https://github.com/Dicklesworthstone/beads_rust/commit/4933e1b)).

10 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.11...v0.1.12).

---

### [`v0.1.11`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.11) -- 2026-01-28 (tag only)

- **`--wrap` flag** for `br blocked` command ([`1652796`](https://github.com/Dicklesworthstone/beads_rust/commit/1652796)).
- **Structured error validation** and error parity tests ([`153aa06`](https://github.com/Dicklesworthstone/beads_rust/commit/153aa06)).
- **Label test isolation** and ID parsing for new output format ([`b9aa3fa`](https://github.com/Dicklesworthstone/beads_rust/commit/b9aa3fa)).

13 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.10...v0.1.11).

---

### [`v0.1.10`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.10) -- 2026-01-28 (tag only)

**Theme:** TOON format and ecosystem integration.

- **TOON output format** for token-optimized serialization ([`b1882b8`](https://github.com/Dicklesworthstone/beads_rust/commit/b1882b8)).
- **Nix flake support** ([`d5e9821`](https://github.com/Dicklesworthstone/beads_rust/commit/d5e9821)).
- **Schema command** and CLI structure improvements ([`9da03ba`](https://github.com/Dicklesworthstone/beads_rust/commit/9da03ba)).
- **VCS integration guide** ([`7596071`](https://github.com/Dicklesworthstone/beads_rust/commit/7596071)).
- **BEADS\_CACHE\_DIR** for monorepo transient file support ([`fc747cb`](https://github.com/Dicklesworthstone/beads_rust/commit/fc747cb)).
- **Text wrap flag** for output and sync/schema test fixes ([`a122c1b`](https://github.com/Dicklesworthstone/beads_rust/commit/a122c1b)).
- **ACFS notification workflow** for lesson registry sync ([`8d5908d`](https://github.com/Dicklesworthstone/beads_rust/commit/8d5908d)).

29 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.9...v0.1.10).

---

### [`v0.1.9`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.9) -- 2026-01-23 (tag only)

- **ID-prefix dot validation** and `--force` skip option ([`6d5d0a1`](https://github.com/Dicklesworthstone/beads_rust/commit/6d5d0a1)).
- **blocked\_issues\_cache** SQL reference fix in `get_ready_issues` ([`27fa5dd`](https://github.com/Dicklesworthstone/beads_rust/commit/27fa5dd)).
- **`--status` flag** on `br create` ([`cac47de`](https://github.com/Dicklesworthstone/beads_rust/commit/cac47de)).
- **CLI output** and sync formatting improvements ([`e58e90b`](https://github.com/Dicklesworthstone/beads_rust/commit/e58e90b)).

14 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.8...v0.1.9).

---

### [`v0.1.8`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.8) -- 2026-01-22 (tag only)

**Theme:** The first major post-launch stabilization and feature push.

- **Rich output foundation** with `rich_rust` integration ([`1cb7051`](https://github.com/Dicklesworthstone/beads_rust/commit/1cb7051), [`d85e89a`](https://github.com/Dicklesworthstone/beads_rust/commit/d85e89a)).
- **Conformance harness** with bd vs br comparison infrastructure ([`55821a2`](https://github.com/Dicklesworthstone/beads_rust/commit/55821a2)).
- **Storage schema expansion** with enhanced field metadata and search indices ([`0e1f869`](https://github.com/Dicklesworthstone/beads_rust/commit/0e1f869)).
- **Auto-import for mutating commands** to prevent data loss ([`24fd16c`](https://github.com/Dicklesworthstone/beads_rust/commit/24fd16c)).
- **Graph depth limiting** to prevent infinite loops in cyclic graphs ([`88e4c96`](https://github.com/Dicklesworthstone/beads_rust/commit/88e4c96)).
- **Self-update** with signature verification and benchmark regression harness ([`22b04e6`](https://github.com/Dicklesworthstone/beads_rust/commit/22b04e6)).
- **Gate columns** and DATETIME type migration ([`7990eae`](https://github.com/Dicklesworthstone/beads_rust/commit/7990eae)).
- **Mermaid format** output for dep graph ([`d85f5e3`](https://github.com/Dicklesworthstone/beads_rust/commit/d85f5e3)).
- **External dependency blocking** and CLI/conformance alignment ([`3547b28`](https://github.com/Dicklesworthstone/beads_rust/commit/3547b28)).
- **Multi-repo support**: added `source_repo` field to model ([`30b668c`](https://github.com/Dicklesworthstone/beads_rust/commit/30b668c)).
- **AI coding skills** auto-installation during install ([`18d3e28`](https://github.com/Dicklesworthstone/beads_rust/commit/18d3e28)).
- **Auto-detect issue prefix** from JSONL during migration ([`3a38b45`](https://github.com/Dicklesworthstone/beads_rust/commit/3a38b45)).
- **MIT License** added ([`8858ab7`](https://github.com/Dicklesworthstone/beads_rust/commit/8858ab7)).

77 commits (the largest release). [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.7...v0.1.8).

---

### [`v0.1.7`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.7) -- 2026-01-18

**Theme:** First broadly usable post-launch release.

- **Bulletproof installer** with fallback to source build ([`f09877d`](https://github.com/Dicklesworthstone/beads_rust/commit/f09877d)).
- **AGENTS.md blurb** detection and management ([`cbd9e95`](https://github.com/Dicklesworthstone/beads_rust/commit/cbd9e95)).
- **macOS CI fix**: use shasum instead of sha256sum ([`d2e6131`](https://github.com/Dicklesworthstone/beads_rust/commit/d2e6131)).
- **Snapshot normalization** for usernames and version numbers in tests ([`0154711`](https://github.com/Dicklesworthstone/beads_rust/commit/0154711)).
- **macos-13** retired runner fix ([`3742b50`](https://github.com/Dicklesworthstone/beads_rust/commit/3742b50)).
- **BASH\_SOURCE** guard for piped execution ([`f978117`](https://github.com/Dicklesworthstone/beads_rust/commit/f978117)).

14 commits. [Full diff](https://github.com/Dicklesworthstone/beads_rust/compare/v0.1.6...v0.1.7).

---

### [`v0.1.0`](https://github.com/Dicklesworthstone/beads_rust/releases/tag/v0.1.0) through [`v0.1.6`](https://github.com/Dicklesworthstone/beads_rust/tree/v0.1.6) -- 2026-01-18

**Launch day.** Seven tags cut in rapid succession (v0.1.0 through v0.1.6) to stabilize CI on the initial public release. The initial draft release includes cross-platform binaries and full CLI compatibility with the original beads. Tags v0.1.1 through v0.1.6 were quick CI/conformance fixes:

- v0.1.0: Initial public release ([`bac6bbf`](https://github.com/Dicklesworthstone/beads_rust/commit/bac6bbf)).
- v0.1.1: Remove accidentally committed build artifacts, consolidate gitignore ([`e58ef54`](https://github.com/Dicklesworthstone/beads_rust/commit/e58ef54)).
- v0.1.2: Skip benchmark tests when `bd` binary unavailable ([`21ec1ad`](https://github.com/Dicklesworthstone/beads_rust/commit/21ec1ad)).
- v0.1.3: Extend `bd` skip check to benchmark\_datasets tests ([`641374e`](https://github.com/Dicklesworthstone/beads_rust/commit/641374e)).
- v0.1.4: Extend skip check to conformance tests ([`9f51da7`](https://github.com/Dicklesworthstone/beads_rust/commit/9f51da7)).
- v0.1.5: Extend skip check to all conformance test files ([`66518a9`](https://github.com/Dicklesworthstone/beads_rust/commit/66518a9)).
- v0.1.6: Fix import order for cargo fmt ([`16c7f36`](https://github.com/Dicklesworthstone/beads_rust/commit/16c7f36)).

---

## Notes for Agents

- **Fastest way to understand a historical capability**: read the capability section above, then open linked commits for the actual implementation.
- **Most important late-history themes**:
  - Sync safety and no-data-loss guarantees.
  - Blocked-cache correctness under multi-agent concurrency.
  - Workspace recovery, doctoring, and real-world failure coverage.
  - Agent-facing output correctness across JSON, TOON, quiet, and routing modes.
- **Binary name**: `br` (never `bd` -- that is the original Go version).
- **Non-invasive rule**: `br` never runs git commands automatically. All VCS operations are the caller's responsibility.
- **Default prefix**: changed from `bd` to `br` in v0.1.29.
