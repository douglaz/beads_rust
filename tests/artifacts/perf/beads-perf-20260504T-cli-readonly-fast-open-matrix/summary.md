# CLI Read-Only Fast-Open Matrix

Date: 2026-05-04

Bead: `beads_rust-72yf.3`

## Objective

Extend the hot read-path proof surface from one CLI command to a representative
matrix of read-heavy commands. The slice also adds an explicit conservative
fallback knob:

```bash
BR_DISABLE_READ_ONLY_FAST_OPEN=1 br list --json --limit 1
```

Truthy values (`1`, `true`, `yes`, `on`) disable current-schema read-only
fast-open and force the locked/direct storage path.

## Opportunity Matrix

| Lever | Impact | Confidence | Effort | Score |
|---|---:|---:|---:|---:|
| Add conservative fast-open kill switch plus CLI read matrix proof | 4 | 4 | 2 | 8.0 |
| Promote `comments list` to safe auto-import read-only probe coverage | 3 | 4 | 1 | 12.0 |

Alien-graveyard mapping: this follows the fallback-safe fast-path pattern. The
optimized path has a direct conservative escape hatch, and the matrix proves the
candidate path against that fallback.

Alien-artifact mapping: the proof obligation is behavioral equivalence under a
stable witness: same workspace, same command args, same stdout bytes. The
failure-mode policy is conservative: disable the optimization and rerun through
the locked path.

## Matrix Commands

- `list --json --limit 5`
- `show <id> --format json`
- `ready --json --limit 5`
- `blocked --json --limit 5`
- `comments list <id> --json`
- `dep list <id> --format json`
- `query run fast-open-p1 --format json`

## Proofs

Routine matrix test:

```bash
env CARGO_TARGET_DIR=/data/tmp/beads_rust_dustypuma_cli_fastopen \
  cargo test --test e2e_read_only_fast_open -- --nocapture
```

This asserts:

- Every matrix command succeeds with `BR_DISABLE_READ_ONLY_FAST_OPEN=1`.
- Every matrix command succeeds through default read-only fast-open.
- Fast-open stdout is byte-identical to conservative-path stdout.
- With `.beads/.write.lock` held, every default fast-open matrix command still
  succeeds.
- With `.beads/.write.lock` held, the conservative path times out and reports
  lock contention.

Perf probe:

```bash
env CARGO_TARGET_DIR=/data/tmp/beads_rust_dustypuma_cli_fastopen \
  cargo test --test e2e_read_only_fast_open \
  cli_read_only_fast_open_matrix_perf_probe -- --ignored --nocapture
```

Result:

```json
{
  "commands": [
    "list_json",
    "show_json",
    "ready_json",
    "blocked_json",
    "comments_json",
    "dep_list_json",
    "query_run_json"
  ],
  "rounds": 5,
  "conservative_total_ns": 792360585,
  "fast_open_total_ns": 611925502,
  "speedup_milli": 1294,
  "equality": "routine matrix test asserts byte-identical stdout per command"
}
```

## Isomorphism Proof

- Ordering preserved: yes. The same command implementations render both paths;
  only startup storage-open strategy differs.
- Tie-breaking unchanged: yes. Sorting and filtering live below the storage
  handle and are shared by both paths.
- Floating-point: N/A.
- RNG seeds: N/A.
- Golden outputs: the matrix asserts byte-identical stdout per command.
- Fallback: `BR_DISABLE_READ_ONLY_FAST_OPEN=1` forces the direct locked path.

## Verification

- `cargo fmt --check`
- `git diff --check`
- `cargo test read_only_fast_open -- --nocapture`
- `cargo test --test e2e_read_only_fast_open -- --nocapture`
- `cargo test --test e2e_read_only_fast_open cli_read_only_fast_open_matrix_perf_probe -- --ignored --nocapture`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `ubs src/main.rs docs/SWARM_SCALE_TUNING.md tests/artifacts/perf/beads-perf-20260504T-cli-readonly-fast-open-matrix/summary.md`
  passed with 0 critical issues.
- `ubs src/main.rs tests/common/cli.rs tests/e2e_read_only_fast_open.rs docs/SWARM_SCALE_TUNING.md tests/artifacts/perf/beads-perf-20260504T-cli-readonly-fast-open-matrix/summary.md`
  exited 1 because UBS inventories existing test-helper `panic!` and
  assertion/unwrap surfaces in test files. Its embedded fmt, clippy, cargo
  check, and test-build subchecks were clean.
