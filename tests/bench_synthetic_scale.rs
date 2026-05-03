//! Synthetic scale-up benchmark suite for stress testing with large datasets.
//!
//! This module generates synthetic datasets (100k+ issues) by expanding patterns
//! from real datasets, then exercises list/search/ready/graph/sync operations at scale.
//!
//! # Usage
//!
//! These tests are opt-in only (long-running stress tests):
//! ```bash
//! BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale -- --ignored --nocapture
//! ```
//!
//! # Metrics Captured
//!
//! - Wall-clock time for each operation
//! - Peak RSS (memory) on Linux
//! - Export/import file sizes
//! - Issue counts and dependency density
//!
//! # Scale Tiers
//!
//! - Small: 10,000 issues (quick sanity check)
//! - Medium: 50,000 issues
//! - Large: 100,000 issues
//! - XLarge: 250,000 issues (very long-running)

#![allow(
    clippy::cast_precision_loss,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::too_many_lines,
    clippy::missing_const_for_fn
)]

mod common;

use beads_rust::model::{Comment, Dependency, DependencyType, Issue, IssueType, Priority, Status};
use beads_rust::util::hex_encode;
use chrono::Utc;
use common::binary_discovery::discover_binaries;
use common::dataset_registry::KnownDataset;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use tempfile::TempDir;

// =============================================================================
// Configuration
// =============================================================================

/// Check if stress tests are enabled.
fn stress_tests_enabled() -> bool {
    std::env::var("BR_E2E_STRESS").is_ok()
}

/// Check if the manual million-issue profile is enabled.
fn million_profile_enabled() -> bool {
    std::env::var("BR_SYNTHETIC_MILLION").is_ok()
}

fn synthetic_seed_from_env(default_seed: u64) -> u64 {
    std::env::var("BR_SYNTHETIC_SEED")
        .ok()
        .and_then(|seed| seed.parse().ok())
        .unwrap_or(default_seed)
}

fn synthetic_evidence_issue_count_from_env(default_count: usize) -> usize {
    std::env::var("BR_SYNTHETIC_EVIDENCE_ISSUES")
        .ok()
        .and_then(|count| count.parse().ok())
        .filter(|count| *count > 0)
        .unwrap_or(default_count)
}

fn synthetic_evidence_output_dir() -> PathBuf {
    std::env::var_os("BR_SYNTHETIC_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/benchmark-results"))
}

/// Scale tier for synthetic datasets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleTier {
    /// 10,000 issues - quick sanity check
    Small,
    /// 50,000 issues - medium stress
    Medium,
    /// 100,000 issues - standard stress test
    Large,
    /// 250,000 issues - extreme stress test
    XLarge,
    /// 1,000,000 issues - manual 256GB+/64-core profile
    Million,
}

impl ScaleTier {
    #[must_use]
    pub const fn issue_count(self) -> usize {
        match self {
            Self::Small => 10_000,
            Self::Medium => 50_000,
            Self::Large => 100_000,
            Self::XLarge => 250_000,
            Self::Million => 1_000_000,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "small_10k",
            Self::Medium => "medium_50k",
            Self::Large => "large_100k",
            Self::XLarge => "xlarge_250k",
            Self::Million => "million_1m",
        }
    }

    /// Target dependency density (deps per issue on average).
    #[must_use]
    pub const fn dependency_density(self) -> f64 {
        match self {
            Self::Small => 0.3,
            Self::Medium | Self::Large => 0.5,
            Self::XLarge => 0.7,
            Self::Million => 0.9,
        }
    }
}

// =============================================================================
// Synthetic Dataset Generator
// =============================================================================

/// Configuration for synthetic dataset generation.
#[derive(Debug, Clone)]
pub struct SyntheticConfig {
    /// Target number of issues
    pub issue_count: usize,
    /// Average dependencies per issue (0.0 - 1.0)
    pub dependency_density: f64,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Base dataset to expand (for realistic patterns)
    pub base_dataset: Option<KnownDataset>,
    /// Number of label names in the synthetic label pool.
    pub label_pool_size: usize,
    /// Minimum labels assigned per issue.
    pub min_labels_per_issue: usize,
    /// Maximum labels assigned per issue.
    pub max_labels_per_issue: usize,
    /// Probability that an issue receives comments.
    pub comment_density: f64,
    /// Maximum comments per commented issue.
    pub max_comments_per_issue: usize,
    /// Simulated agent identities available for claims and comments.
    pub simulated_agent_count: usize,
    /// Probability that an issue is claimed by a simulated agent.
    pub claim_density: f64,
    /// Bias dependencies toward low-numbered hub issues for skewed DAG profiles.
    pub dag_skew: f64,
}

impl SyntheticConfig {
    #[must_use]
    pub fn from_tier(tier: ScaleTier) -> Self {
        Self {
            issue_count: tier.issue_count(),
            dependency_density: tier.dependency_density(),
            seed: 42, // Reproducible by default
            base_dataset: Some(KnownDataset::BeadsRust),
            label_pool_size: 64,
            min_labels_per_issue: 0,
            max_labels_per_issue: 4,
            comment_density: 0.15,
            max_comments_per_issue: 3,
            simulated_agent_count: 10_000,
            claim_density: 0.05,
            dag_skew: 1.25,
        }
    }

    #[must_use]
    pub const fn ci_profile(seed: u64) -> Self {
        Self {
            issue_count: 256,
            dependency_density: 0.4,
            seed,
            base_dataset: None,
            label_pool_size: 12,
            min_labels_per_issue: 0,
            max_labels_per_issue: 3,
            comment_density: 0.25,
            max_comments_per_issue: 2,
            simulated_agent_count: 16,
            claim_density: 0.2,
            dag_skew: 0.8,
        }
    }

    #[must_use]
    pub const fn million_agent_profile(seed: u64) -> Self {
        Self {
            issue_count: ScaleTier::Million.issue_count(),
            dependency_density: ScaleTier::Million.dependency_density(),
            seed,
            base_dataset: Some(KnownDataset::BeadsRust),
            label_pool_size: 512,
            min_labels_per_issue: 1,
            max_labels_per_issue: 6,
            comment_density: 0.2,
            max_comments_per_issue: 4,
            simulated_agent_count: 10_000,
            claim_density: 0.08,
            dag_skew: 1.8,
        }
    }

    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    #[must_use]
    pub const fn with_issue_count(mut self, issue_count: usize) -> Self {
        self.issue_count = issue_count;
        self
    }

    #[must_use]
    pub const fn with_label_distribution(
        mut self,
        label_pool_size: usize,
        min_labels_per_issue: usize,
        max_labels_per_issue: usize,
    ) -> Self {
        self.label_pool_size = label_pool_size;
        self.min_labels_per_issue = min_labels_per_issue;
        self.max_labels_per_issue = max_labels_per_issue;
        self
    }

    #[must_use]
    pub const fn with_comment_distribution(
        mut self,
        comment_density: f64,
        max_comments_per_issue: usize,
    ) -> Self {
        self.comment_density = comment_density;
        self.max_comments_per_issue = max_comments_per_issue;
        self
    }

    #[must_use]
    pub const fn with_agent_distribution(
        mut self,
        simulated_agent_count: usize,
        claim_density: f64,
    ) -> Self {
        self.simulated_agent_count = simulated_agent_count;
        self.claim_density = claim_density;
        self
    }

    #[must_use]
    pub const fn with_dag_skew(mut self, dag_skew: f64) -> Self {
        self.dag_skew = dag_skew;
        self
    }
}

/// Metrics from synthetic dataset generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetrics {
    /// Actual issue count generated
    pub issue_count: usize,
    /// Actual dependency count generated
    pub dependency_count: usize,
    /// Total labels assigned across all issues.
    pub label_assignment_count: usize,
    /// Total comments generated across all issues.
    pub comment_count: usize,
    /// Number of simulated agent identities in the corpus.
    pub simulated_agent_count: usize,
    /// Number of claimed issues assigned to simulated agents.
    pub claim_count: usize,
    /// Generation duration
    pub generation_ms: u128,
    /// JSONL file size in bytes
    pub jsonl_size_bytes: u64,
    /// Byte count predicted by the generator while streaming JSONL.
    pub expected_jsonl_size_bytes: u64,
    /// DB file size after rebuild
    pub db_size_bytes: u64,
    /// SHA-256 hash of generated issues.jsonl.
    pub content_hash: String,
    /// Health checks recorded after import/rebuild.
    pub health: GenerationHealth,
}

/// Health evidence for a generated corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationHealth {
    /// Every JSONL line parsed as an Issue.
    pub jsonl_valid: bool,
    /// Number of valid JSONL issue records.
    pub jsonl_issue_count: usize,
    /// `br sync --import-only --json` succeeded.
    pub sync_import_ok: bool,
    /// `br doctor --json` succeeded after import.
    pub doctor_ok: bool,
    /// `br sync --status --json` reported no dirty DB/JSONL divergence.
    pub sync_status_clean: bool,
}

/// Reproducibility manifest for a generated synthetic corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticCorpusManifest {
    pub schema_version: String,
    pub generator: String,
    pub generated_at: String,
    pub config: SyntheticConfigSnapshot,
    pub metrics: GenerationMetrics,
    pub reproduction_command: String,
}

/// Serializable subset of SyntheticConfig.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticConfigSnapshot {
    pub issue_count: usize,
    pub dependency_density: f64,
    pub seed: u64,
    pub base_dataset: Option<String>,
    pub label_pool_size: usize,
    pub min_labels_per_issue: usize,
    pub max_labels_per_issue: usize,
    pub comment_density: f64,
    pub max_comments_per_issue: usize,
    pub simulated_agent_count: usize,
    pub claim_density: f64,
    pub dag_skew: f64,
}

/// A generated synthetic dataset in an isolated workspace.
pub struct SyntheticDataset {
    pub temp_dir: TempDir,
    pub root: PathBuf,
    pub beads_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub config: SyntheticConfig,
    pub metrics: GenerationMetrics,
}

impl SyntheticDataset {
    /// Generate a synthetic dataset based on the config.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary workspace or any CLI command fails.
    pub fn generate(config: SyntheticConfig, br_path: &Path) -> std::io::Result<Self> {
        let start = Instant::now();
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();
        let beads_dir = root.join(".beads");
        let manifest_path = root.join("synthetic-corpus-manifest.json");

        // Create minimal git scaffold
        fs::create_dir_all(root.join(".git"))?;
        fs::write(root.join(".git").join("HEAD"), "ref: refs/heads/main\n")?;

        // Initialize beads
        let init_output = Command::new(br_path) // ubs:ignore - benchmark harness executes only discovered br binaries
            .args(["init"])
            .current_dir(&root)
            .output()?;

        if !init_output.status.success() {
            return Err(std::io::Error::other(format!(
                "br init failed: {}",
                String::from_utf8_lossy(&init_output.stderr)
            )));
        }

        let generated = write_synthetic_jsonl(&config, &beads_dir.join("issues.jsonl"))?;
        let sync_import_ok = run_br_status(
            br_path,
            ["sync", "--import-only", "--json"],
            &root,
            "br sync --import-only",
        )?;
        let doctor_ok = run_br_status(br_path, ["doctor", "--json"], &root, "br doctor")?;
        let sync_status_clean = sync_status_is_clean(br_path, &root)?;
        let jsonl_health = validate_generated_jsonl(&beads_dir.join("issues.jsonl"))?;

        let generation_ms = start.elapsed().as_millis();
        let db_path = beads_dir.join("beads.db");
        let db_size_bytes = fs::metadata(&db_path).map_or(0, |m| m.len());

        let metrics = GenerationMetrics {
            issue_count: generated.issue_count,
            dependency_count: generated.dependency_count,
            label_assignment_count: generated.label_assignment_count,
            comment_count: generated.comment_count,
            simulated_agent_count: config.simulated_agent_count,
            claim_count: generated.claim_count,
            generation_ms,
            jsonl_size_bytes: generated.jsonl_size_bytes,
            expected_jsonl_size_bytes: generated.expected_jsonl_size_bytes,
            db_size_bytes,
            content_hash: generated.content_hash,
            health: GenerationHealth {
                jsonl_valid: jsonl_health.valid,
                jsonl_issue_count: jsonl_health.issue_count,
                sync_import_ok,
                doctor_ok,
                sync_status_clean,
            },
        };

        let manifest = SyntheticCorpusManifest {
            schema_version: "br.synthetic-corpus.v1".to_string(),
            generator: "bench_synthetic_scale::write_synthetic_jsonl".to_string(),
            generated_at: Utc::now().to_rfc3339(),
            config: SyntheticConfigSnapshot::from(&config),
            metrics: metrics.clone(),
            reproduction_command: reproduction_command_for(&config),
        };
        write_json_pretty(&manifest_path, &manifest)?;

        Ok(Self {
            temp_dir,
            root,
            beads_dir,
            manifest_path,
            config,
            metrics,
        })
    }

    /// Get workspace root for command execution.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.root
    }
}

impl From<&SyntheticConfig> for SyntheticConfigSnapshot {
    fn from(config: &SyntheticConfig) -> Self {
        Self {
            issue_count: config.issue_count,
            dependency_density: config.dependency_density,
            seed: config.seed,
            base_dataset: config
                .base_dataset
                .map(|dataset| dataset.name().to_string()),
            label_pool_size: config.label_pool_size,
            min_labels_per_issue: config.min_labels_per_issue,
            max_labels_per_issue: config.max_labels_per_issue,
            comment_density: config.comment_density,
            max_comments_per_issue: config.max_comments_per_issue,
            simulated_agent_count: config.simulated_agent_count,
            claim_density: config.claim_density,
            dag_skew: config.dag_skew,
        }
    }
}

struct GeneratedCorpusStats {
    issue_count: usize,
    dependency_count: usize,
    label_assignment_count: usize,
    comment_count: usize,
    claim_count: usize,
    jsonl_size_bytes: u64,
    expected_jsonl_size_bytes: u64,
    content_hash: String,
}

struct JsonlHealth {
    valid: bool,
    issue_count: usize,
}

fn write_synthetic_jsonl(
    config: &SyntheticConfig,
    jsonl_path: &Path,
) -> std::io::Result<GeneratedCorpusStats> {
    let file = File::create(jsonl_path)?;
    let mut writer = BufWriter::new(file);
    let mut hasher = Sha256::new();
    let mut rng = StdRng::seed_from_u64(config.seed);

    let mut stats = GeneratedCorpusStats {
        issue_count: 0,
        dependency_count: 0,
        label_assignment_count: 0,
        comment_count: 0,
        claim_count: 0,
        jsonl_size_bytes: 0,
        expected_jsonl_size_bytes: 0,
        content_hash: String::new(),
    };

    for index in 0..config.issue_count {
        let issue = generate_synthetic_issue(config, &mut rng, index, &mut stats);
        let line = serde_json::to_vec(&issue)?;
        writer.write_all(&line)?;
        writer.write_all(b"\n")?;
        hasher.update(&line);
        hasher.update(b"\n");
        stats.expected_jsonl_size_bytes = stats
            .expected_jsonl_size_bytes
            .saturating_add(u64::try_from(line.len()).unwrap_or(u64::MAX))
            .saturating_add(1);
        stats.issue_count += 1;

        if stats.issue_count.is_multiple_of(100_000) {
            eprintln!(
                "  Streamed {}/{} synthetic issues...",
                stats.issue_count, config.issue_count
            );
        }
    }
    writer.flush()?;

    stats.jsonl_size_bytes = fs::metadata(jsonl_path).map_or(0, |metadata| metadata.len());
    stats.content_hash = hex_encode(&hasher.finalize());
    Ok(stats)
}

fn generate_synthetic_issue(
    config: &SyntheticConfig,
    rng: &mut StdRng,
    index: usize,
    stats: &mut GeneratedCorpusStats,
) -> Issue {
    let id = synthetic_issue_id(index);
    let created_at = synthetic_timestamp(index);
    let labels = generate_labels(config, rng);
    let dependencies = generate_dependencies(config, rng, index, &id, created_at);
    let comments = generate_comments(config, rng, index, &id, created_at, stats.comment_count);
    let assignee = choose_claimed_agent(config, rng);

    stats.dependency_count += dependencies.len();
    stats.label_assignment_count += labels.len();
    stats.comment_count += comments.len();
    if assignee.is_some() {
        stats.claim_count += 1;
    }

    Issue {
        id,
        title: generate_title(rng, index),
        description: Some(format!(
            "Synthetic scale corpus issue {index}; seed={}; agents={}; generated for br large-workspace benchmarking.",
            config.seed, config.simulated_agent_count
        )),
        status: if assignee.is_some() {
            Status::InProgress
        } else {
            Status::Open
        },
        priority: Priority(rng.random_range(0..=4)),
        issue_type: synthetic_issue_type(rng),
        assignee,
        created_at,
        created_by: Some(synthetic_agent_name(
            config.seed,
            index % effective_agent_count(config),
        )),
        updated_at: created_at,
        source_repo: Some("synthetic-swarm-corpus".to_string()),
        compaction_level: Some(0),
        original_size: Some(0),
        labels,
        dependencies,
        comments,
        ..Issue::default()
    }
}

fn synthetic_issue_id(index: usize) -> String {
    format!("synth-{index:08x}")
}

fn synthetic_timestamp(index: usize) -> chrono::DateTime<Utc> {
    let base = chrono::DateTime::<Utc>::UNIX_EPOCH;
    base + chrono::Duration::seconds(usize_to_i64(index % 86_400))
}

fn synthetic_issue_type(rng: &mut StdRng) -> IssueType {
    match rng.random_range(0..10) {
        0..=5 => IssueType::Task,
        6..=7 => IssueType::Bug,
        8 => IssueType::Feature,
        _ => IssueType::Chore,
    }
}

fn effective_agent_count(config: &SyntheticConfig) -> usize {
    config.simulated_agent_count.max(1)
}

fn synthetic_agent_name(seed: u64, index: usize) -> String {
    format!("agent-{seed:016x}-{index:05}")
}

fn choose_claimed_agent(config: &SyntheticConfig, rng: &mut StdRng) -> Option<String> {
    if config.claim_density <= 0.0 {
        return None;
    }
    if rng.random_range(0.0..1.0) >= config.claim_density.min(1.0) {
        return None;
    }
    Some(synthetic_agent_name(
        config.seed,
        rng.random_range(0..effective_agent_count(config)),
    ))
}

fn generate_labels(config: &SyntheticConfig, rng: &mut StdRng) -> Vec<String> {
    if config.label_pool_size == 0 || config.max_labels_per_issue == 0 {
        return Vec::new();
    }

    let min = config.min_labels_per_issue.min(config.max_labels_per_issue);
    let max = config.max_labels_per_issue.max(min);
    let label_count = rng.random_range(min..=max).min(config.label_pool_size);
    let mut labels = BTreeSet::new();

    while labels.len() < label_count {
        labels.insert(format!(
            "label-{:03}",
            rng.random_range(0..config.label_pool_size)
        ));
    }

    labels.into_iter().collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn generate_dependencies(
    config: &SyntheticConfig,
    rng: &mut StdRng,
    index: usize,
    issue_id: &str,
    created_at: chrono::DateTime<Utc>,
) -> Vec<Dependency> {
    if index == 0 || config.dependency_density <= 0.0 {
        return Vec::new();
    }

    let guaranteed = config.dependency_density.floor() as usize;
    let fractional = config.dependency_density.fract();
    let dep_count = guaranteed + usize::from(rng.random_range(0.0..1.0) < fractional);
    let dep_count = dep_count.min(index);
    let mut targets = BTreeSet::new();

    while targets.len() < dep_count {
        targets.insert(skewed_dependency_target(config, rng, index));
    }

    targets
        .into_iter()
        .map(|target| Dependency {
            issue_id: issue_id.to_string(),
            depends_on_id: synthetic_issue_id(target),
            dep_type: DependencyType::Blocks,
            created_at,
            created_by: Some("synthetic-corpus-generator".to_string()),
            metadata: Some(format!(
                "{{\"seed\":{},\"dag_skew\":{}}}",
                config.seed, config.dag_skew
            )),
            thread_id: None,
        })
        .collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn skewed_dependency_target(
    config: &SyntheticConfig,
    rng: &mut StdRng,
    upper_bound: usize,
) -> usize {
    if upper_bound <= 1 {
        return 0;
    }

    let skew = config.dag_skew.max(0.0);
    let sample = rng.random_range(0.0..1.0_f64).powf(1.0 + skew);
    ((sample * upper_bound as f64).floor() as usize).min(upper_bound - 1)
}

fn generate_comments(
    config: &SyntheticConfig,
    rng: &mut StdRng,
    index: usize,
    issue_id: &str,
    created_at: chrono::DateTime<Utc>,
    next_comment_id: usize,
) -> Vec<Comment> {
    if config.comment_density <= 0.0 || config.max_comments_per_issue == 0 {
        return Vec::new();
    }
    if rng.random_range(0.0..1.0) >= config.comment_density.min(1.0) {
        return Vec::new();
    }

    let comment_count = rng.random_range(1..=config.max_comments_per_issue);
    (0..comment_count)
        .map(|offset| Comment {
            id: usize_to_i64(next_comment_id + offset + 1),
            issue_id: issue_id.to_string(),
            author: synthetic_agent_name(
                config.seed,
                rng.random_range(0..effective_agent_count(config)),
            ),
            body: format!(
                "Synthetic agent note {offset} for issue {index}; seed={}; reproducible benchmark corpus.",
                config.seed
            ),
            created_at: created_at + chrono::Duration::seconds(usize_to_i64(offset + 1)),
        })
        .collect()
}

fn usize_to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn validate_generated_jsonl(jsonl_path: &Path) -> std::io::Result<JsonlHealth> {
    let content = fs::read_to_string(jsonl_path)?;
    let mut issue_count = 0;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        serde_json::from_str::<Issue>(line).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("generated JSONL issue line is invalid: {err}"),
            )
        })?;
        issue_count += 1;
    }

    Ok(JsonlHealth {
        valid: true,
        issue_count,
    })
}

fn run_br_status<const N: usize>(
    br_path: &Path,
    args: [&str; N],
    workspace: &Path,
    label: &str,
) -> std::io::Result<bool> {
    let output = Command::new(br_path) // ubs:ignore - benchmark harness executes only discovered br binaries
        .args(args)
        .current_dir(workspace)
        .env("NO_COLOR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        return Ok(true);
    }

    Err(std::io::Error::other(format!(
        "{label} failed: stdout={}; stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

fn sync_status_is_clean(br_path: &Path, workspace: &Path) -> std::io::Result<bool> {
    let output = Command::new(br_path) // ubs:ignore - benchmark harness executes only discovered br binaries
        .args(["sync", "--status", "--json"])
        .current_dir(workspace)
        .env("NO_COLOR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "br sync --status failed: stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid br sync --status JSON: {err}"),
        )
    })?;

    Ok(value
        .get("dirty_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(u64::MAX)
        == 0
        && !value
            .get("jsonl_newer")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
        && !value
            .get("db_newer")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true))
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}

fn reproduction_command_for(config: &SyntheticConfig) -> String {
    if config.issue_count >= ScaleTier::Million.issue_count() {
        format!(
            "BR_E2E_STRESS=1 BR_SYNTHETIC_MILLION=1 BR_SYNTHETIC_SEED={} cargo test --test bench_synthetic_scale stress_synthetic_million -- --ignored --nocapture",
            config.seed
        )
    } else {
        format!(
            "BR_E2E_STRESS=1 BR_SYNTHETIC_SEED={} cargo test --test bench_synthetic_scale -- --ignored --nocapture",
            config.seed
        )
    }
}

/// Generate a realistic-looking issue title.
fn generate_title(rng: &mut StdRng, index: usize) -> String {
    let prefixes = [
        "Add",
        "Fix",
        "Update",
        "Refactor",
        "Implement",
        "Remove",
        "Improve",
        "Optimize",
        "Document",
        "Test",
        "Review",
        "Debug",
        "Cleanup",
        "Migrate",
        "Configure",
    ];

    let subjects = [
        "authentication flow",
        "database connection",
        "API endpoint",
        "user interface",
        "error handling",
        "logging system",
        "configuration",
        "test coverage",
        "documentation",
        "performance",
        "security",
        "caching",
        "validation",
        "serialization",
        "routing",
    ];

    let prefix = prefixes[rng.random_range(0..prefixes.len())];
    let subject = subjects[rng.random_range(0..subjects.len())];

    format!("{prefix} {subject} (#{index})")
}

// =============================================================================
// Benchmark Metrics
// =============================================================================

/// Metrics for a single benchmark operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Operation name
    pub operation: String,
    /// Wall-clock duration in milliseconds
    pub duration_ms: u128,
    /// Peak RSS in bytes (Linux only)
    pub peak_rss_bytes: Option<u64>,
    /// Whether the operation succeeded
    pub success: bool,
    /// Output size in bytes
    pub output_size_bytes: usize,
    /// SHA-256 of stdout for output identity evidence.
    pub stdout_sha256: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Full benchmark results for a synthetic dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticBenchmark {
    /// Scale tier name
    pub tier: String,
    /// Dataset configuration used for reproducibility.
    pub config: SyntheticConfigSnapshot,
    /// Dataset generation metrics
    pub generation: GenerationMetrics,
    /// Operation benchmarks
    pub operations: Vec<OperationMetrics>,
    /// Summary statistics
    pub summary: BenchmarkSummary,
    /// `br` binary path measured by the benchmark.
    pub br_binary_path: String,
    /// Command that can reproduce this benchmark profile.
    pub reproduction_command: String,
    /// Timestamp
    pub timestamp: String,
}

/// Summary of benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total benchmark duration (including generation)
    pub total_duration_ms: u128,
    /// Average operation duration
    pub avg_operation_ms: u128,
    /// Slowest operation
    pub slowest_operation: String,
    /// Slowest operation duration
    pub slowest_duration_ms: u128,
    /// Operations per second (throughput)
    pub ops_per_second: f64,
    /// Issues per second (for list operations)
    pub issues_per_second: Option<f64>,
}

// =============================================================================
// Benchmark Runner
// =============================================================================

const SYNTHETIC_FULL_GRAPH_MAX_ISSUES: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticWorkloadSpec {
    operation: &'static str,
    args: Vec<String>,
}

fn synthetic_graph_workloads(issue_count: usize) -> Vec<SyntheticWorkloadSpec> {
    let hot_hub = synthetic_issue_id(0);
    let hot_leaf = synthetic_issue_id(issue_count.saturating_sub(1));

    let mut workloads = vec![
        SyntheticWorkloadSpec {
            operation: "graph_hot_hub",
            args: vec!["graph".to_string(), hot_hub, "--json".to_string()],
        },
        SyntheticWorkloadSpec {
            operation: "dep_tree_hot_leaf",
            args: vec![
                "dep".to_string(),
                "tree".to_string(),
                hot_leaf,
                "--direction".to_string(),
                "down".to_string(),
                "--max-depth".to_string(),
                "12".to_string(),
                "--json".to_string(),
            ],
        },
    ];

    if issue_count <= SYNTHETIC_FULL_GRAPH_MAX_ISSUES {
        workloads.push(SyntheticWorkloadSpec {
            operation: "graph_all_components",
            args: vec![
                "graph".to_string(),
                "--all".to_string(),
                "--json".to_string(),
            ],
        });
    }

    workloads
}

/// Run a command and capture metrics.
fn run_operation(
    br_path: &Path,
    args: &[&str],
    workspace: &Path,
    operation: &str,
) -> OperationMetrics {
    let start = Instant::now();

    let output = run_measured_br_command(br_path, args, workspace);

    let duration = start.elapsed();

    match output {
        Ok(out) => {
            let MeasuredCommandOutput {
                stdout,
                stderr,
                success,
                peak_rss_bytes,
            } = out;
            let stdout_sha256 = if success {
                Some(sha256_hex(&stdout))
            } else {
                None
            };
            let error = if success { None } else { Some(stderr) };

            OperationMetrics {
                operation: operation.to_string(),
                duration_ms: duration.as_millis(),
                peak_rss_bytes,
                success,
                output_size_bytes: stdout.len(),
                stdout_sha256,
                error,
            }
        }
        Err(e) => OperationMetrics {
            operation: operation.to_string(),
            duration_ms: duration.as_millis(),
            peak_rss_bytes: None,
            success: false,
            output_size_bytes: 0,
            stdout_sha256: None,
            error: Some(e.to_string()),
        },
    }
}

struct MeasuredCommandOutput {
    stdout: Vec<u8>,
    stderr: String,
    success: bool,
    peak_rss_bytes: Option<u64>,
}

fn run_measured_br_command(
    br_path: &Path,
    args: &[&str],
    workspace: &Path,
) -> std::io::Result<MeasuredCommandOutput> {
    if Path::new("/usr/bin/time").is_file() {
        let output = Command::new("/usr/bin/time") // ubs:ignore - benchmark harness intentionally invokes GNU time for child RSS
            .arg("-v")
            .arg(br_path)
            .args(args)
            .current_dir(workspace)
            .env("NO_COLOR", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Ok(MeasuredCommandOutput {
            stdout: output.stdout,
            peak_rss_bytes: parse_time_max_rss_bytes(&stderr),
            stderr,
            success: output.status.success(),
        });
    }

    let output = Command::new(br_path) // ubs:ignore - benchmark harness executes only discovered br binaries
        .args(args)
        .current_dir(workspace)
        .env("NO_COLOR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(MeasuredCommandOutput {
        stdout: output.stdout,
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
        peak_rss_bytes: None,
    })
}

fn parse_time_max_rss_bytes(stderr: &str) -> Option<u64> {
    stderr.lines().find_map(|line| {
        let kb = line
            .trim_start()
            .strip_prefix("Maximum resident set size (kbytes):")?
            .trim()
            .parse::<u64>()
            .ok()?;
        Some(kb.saturating_mul(1024))
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

/// Run full benchmark suite on a synthetic dataset.
fn benchmark_synthetic(dataset: &SyntheticDataset, br_path: &Path) -> SyntheticBenchmark {
    let start = Instant::now();
    let mut operations = Vec::new();
    let workspace = dataset.workspace_root();

    // Read operations
    operations.push(run_operation(
        br_path,
        &["list", "--json"],
        workspace,
        "list",
    ));
    operations.push(run_operation(
        br_path,
        &["list", "--status=open", "--json"],
        workspace,
        "list_open",
    ));
    operations.push(run_operation(
        br_path,
        &["ready", "--json"],
        workspace,
        "ready",
    ));
    operations.push(run_operation(
        br_path,
        &["stats", "--json"],
        workspace,
        "stats",
    ));
    operations.push(run_operation(
        br_path,
        &["search", "test", "--json"],
        workspace,
        "search",
    ));
    operations.push(run_operation(
        br_path,
        &["blocked", "--json"],
        workspace,
        "blocked",
    ));

    for workload in synthetic_graph_workloads(dataset.config.issue_count) {
        let args: Vec<&str> = workload.args.iter().map(String::as_str).collect();
        operations.push(run_operation(br_path, &args, workspace, workload.operation));
    }

    // Export operation
    operations.push(run_operation(
        br_path,
        &["sync", "--flush-only", "--json"],
        workspace,
        "sync_flush",
    ));

    // Calculate summary
    let total_duration_ms = start.elapsed().as_millis();
    let successful_ops: Vec<_> = operations.iter().filter(|o| o.success).collect();

    let avg_operation_ms = if successful_ops.is_empty() {
        0
    } else {
        successful_ops.iter().map(|o| o.duration_ms).sum::<u128>() / successful_ops.len() as u128
    };

    let (slowest_operation, slowest_duration_ms) =
        operations.iter().max_by_key(|o| o.duration_ms).map_or_else(
            || ("none".to_string(), 0),
            |o| (o.operation.clone(), o.duration_ms),
        );

    let ops_per_second = if total_duration_ms > 0 {
        (operations.len() as f64 * 1000.0) / total_duration_ms as f64
    } else {
        0.0
    };

    // Calculate issues/second for list operation
    let issues_per_second = operations
        .iter()
        .find(|o| o.operation == "list" && o.success)
        .map(|o| {
            if o.duration_ms > 0 {
                (dataset.metrics.issue_count as f64 * 1000.0) / o.duration_ms as f64
            } else {
                0.0
            }
        });

    let summary = BenchmarkSummary {
        total_duration_ms,
        avg_operation_ms,
        slowest_operation,
        slowest_duration_ms,
        ops_per_second,
        issues_per_second,
    };

    let timestamp = chrono::Utc::now().to_rfc3339();

    SyntheticBenchmark {
        tier: format!(
            "synthetic_{}",
            match dataset.config.issue_count {
                n if n <= 10_000 => "small",
                n if n <= 50_000 => "medium",
                n if n <= 100_000 => "large",
                n if n < 1_000_000 => "xlarge",
                _ => "million",
            }
        ),
        config: SyntheticConfigSnapshot::from(&dataset.config),
        generation: dataset.metrics.clone(),
        operations,
        summary,
        br_binary_path: br_path.display().to_string(),
        reproduction_command: reproduction_command_for(&dataset.config),
        timestamp,
    }
}

/// Print benchmark results to stdout.
fn print_benchmark(benchmark: &SyntheticBenchmark) {
    let sep = "=".repeat(80);
    let dash = "-".repeat(80);

    println!("\n{sep}");
    println!("Synthetic Benchmark: {}", benchmark.tier);
    println!("{sep}");

    // Generation metrics
    let generation = &benchmark.generation;
    println!(
        "Dataset: {} issues, {} dependencies, {} labels, {} comments, {} claims across {} agents ({:.1} KB JSONL, {:.1} KB DB)",
        generation.issue_count,
        generation.dependency_count,
        generation.label_assignment_count,
        generation.comment_count,
        generation.claim_count,
        generation.simulated_agent_count,
        generation.jsonl_size_bytes as f64 / 1024.0,
        generation.db_size_bytes as f64 / 1024.0
    );
    println!("Generation time: {}ms", generation.generation_ms);
    println!("JSONL hash: {}", generation.content_hash);
    println!(
        "Health: jsonl_valid={} sync_import_ok={} doctor_ok={} sync_status_clean={}",
        generation.health.jsonl_valid,
        generation.health.sync_import_ok,
        generation.health.doctor_ok,
        generation.health.sync_status_clean
    );
    println!("{dash}");

    // Operations
    println!(
        "{:<20} {:>12} {:>12} {:>12} {:>10}",
        "Operation", "Duration(ms)", "Output(KB)", "RSS(MB)", "Status"
    );
    println!("{dash}");

    for op in &benchmark.operations {
        let status = if op.success { "OK" } else { "FAIL" };
        let output_kb = op.output_size_bytes as f64 / 1024.0;
        let rss_mb = op.peak_rss_bytes.map_or_else(
            || "n/a".to_string(),
            |bytes| format!("{:.1}", bytes as f64 / (1024.0 * 1024.0)),
        );
        println!(
            "{:<20} {:>12} {:>12.1} {:>12} {:>10}",
            op.operation, op.duration_ms, output_kb, rss_mb, status
        );
    }

    // Summary
    let sum = &benchmark.summary;
    println!("{dash}");
    println!("Total duration: {}ms", sum.total_duration_ms);
    println!("Avg operation: {}ms", sum.avg_operation_ms);
    println!(
        "Slowest: {} ({}ms)",
        sum.slowest_operation, sum.slowest_duration_ms
    );
    if let Some(ips) = sum.issues_per_second {
        println!("List throughput: {:.0} issues/second", ips);
    }
    println!();
}

/// Write benchmark results to JSON file.
fn write_benchmark_json(
    benchmarks: &[SyntheticBenchmark],
    output_path: &Path,
) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, benchmarks)?;
    Ok(())
}

fn evidence_reproduction_command_for(config: &SyntheticConfig) -> String {
    format!(
        "BR_E2E_STRESS=1 BR_SYNTHETIC_SEED={} BR_SYNTHETIC_EVIDENCE_ISSUES={} cargo test --test bench_synthetic_scale stress_synthetic_evidence_profile -- --ignored --nocapture",
        config.seed, config.issue_count
    )
}

// =============================================================================
// Tests
// =============================================================================

/// Bounded evidence profile for streaming-output RSS/latency artifacts.
/// Env gate: BR_E2E_STRESS=1
#[test]
#[ignore = "stress test: BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale stress_synthetic_evidence_profile -- --ignored --nocapture"]
fn stress_synthetic_evidence_profile() {
    if !stress_tests_enabled() {
        eprintln!("Skipping stress test (set BR_E2E_STRESS=1 to enable)");
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");
    let seed = synthetic_seed_from_env(20_260_503);
    let issue_count = synthetic_evidence_issue_count_from_env(1_024);
    let config = SyntheticConfig::ci_profile(seed)
        .with_issue_count(issue_count)
        .with_label_distribution(32, 1, 4)
        .with_comment_distribution(0.2, 3)
        .with_agent_distribution(10_000, 0.08)
        .with_dag_skew(1.5);

    eprintln!(
        "Generating bounded synthetic evidence dataset ({} issues, {} simulated agents)...",
        config.issue_count, config.simulated_agent_count
    );
    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("Failed to generate synthetic evidence dataset");

    eprintln!("Running bounded evidence benchmarks...");
    let mut benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    benchmark.tier = "synthetic_evidence".to_string();
    benchmark.reproduction_command = evidence_reproduction_command_for(&dataset.config);
    let failed_operations = benchmark
        .operations
        .iter()
        .filter(|operation| !operation.success)
        .map(|operation| operation.operation.as_str())
        .collect::<Vec<_>>();
    assert!(
        failed_operations.is_empty(),
        "bounded evidence profile should have no failed operations: {failed_operations:?}"
    );
    print_benchmark(&benchmark);

    let output_dir = synthetic_evidence_output_dir();
    fs::create_dir_all(&output_dir).expect("create evidence output dir");
    let result_path = output_dir.join("synthetic_evidence_latest.json");
    write_benchmark_json(&[benchmark], &result_path).expect("write evidence benchmark");
    let manifest_path = output_dir.join("synthetic-corpus-manifest.json");
    fs::copy(&dataset.manifest_path, &manifest_path).expect("persist corpus manifest");

    println!("Evidence benchmark written to: {}", result_path.display());
    println!("Corpus manifest written to: {}", manifest_path.display());
}

/// Small scale synthetic benchmark (10k issues).
/// Env gate: BR_E2E_STRESS=1
#[test]
#[ignore = "stress test: BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale -- --ignored"]
fn stress_synthetic_small() {
    if !stress_tests_enabled() {
        eprintln!("Skipping stress test (set BR_E2E_STRESS=1 to enable)");
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");

    println!("\n=== Synthetic Scale-Up Benchmark: Small (10K) ===\n");

    let config = SyntheticConfig::from_tier(ScaleTier::Small);
    eprintln!(
        "Generating synthetic dataset ({} issues)...",
        config.issue_count
    );

    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("Failed to generate synthetic dataset");

    eprintln!("Running benchmarks...");
    let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    print_benchmark(&benchmark);

    // Write results
    let output_dir = PathBuf::from("target/benchmark-results");
    fs::create_dir_all(&output_dir).expect("create output dir");
    let output_path = output_dir.join("synthetic_small_latest.json");
    write_benchmark_json(&[benchmark], &output_path).expect("write results");
    println!("Results written to: {}", output_path.display());
}

/// Medium scale synthetic benchmark (50k issues).
/// Env gate: BR_E2E_STRESS=1
#[test]
#[ignore = "stress test: BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale -- --ignored"]
fn stress_synthetic_medium() {
    if !stress_tests_enabled() {
        eprintln!("Skipping stress test (set BR_E2E_STRESS=1 to enable)");
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");

    println!("\n=== Synthetic Scale-Up Benchmark: Medium (50K) ===\n");

    let config = SyntheticConfig::from_tier(ScaleTier::Medium);
    eprintln!(
        "Generating synthetic dataset ({} issues)...",
        config.issue_count
    );

    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("Failed to generate synthetic dataset");

    eprintln!("Running benchmarks...");
    let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    print_benchmark(&benchmark);

    // Write results
    let output_dir = PathBuf::from("target/benchmark-results");
    fs::create_dir_all(&output_dir).expect("create output dir");
    let output_path = output_dir.join("synthetic_medium_latest.json");
    write_benchmark_json(&[benchmark], &output_path).expect("write results");
    println!("Results written to: {}", output_path.display());
}

/// Large scale synthetic benchmark (100k issues).
/// Env gate: BR_E2E_STRESS=1
#[test]
#[ignore = "stress test: BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale -- --ignored"]
fn stress_synthetic_large() {
    if !stress_tests_enabled() {
        eprintln!("Skipping stress test (set BR_E2E_STRESS=1 to enable)");
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");

    println!("\n=== Synthetic Scale-Up Benchmark: Large (100K) ===\n");

    let config = SyntheticConfig::from_tier(ScaleTier::Large);
    eprintln!(
        "Generating synthetic dataset ({} issues)...",
        config.issue_count
    );

    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("Failed to generate synthetic dataset");

    eprintln!("Running benchmarks...");
    let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    print_benchmark(&benchmark);

    // Write results
    let output_dir = PathBuf::from("target/benchmark-results");
    fs::create_dir_all(&output_dir).expect("create output dir");
    let output_path = output_dir.join("synthetic_large_latest.json");
    write_benchmark_json(&[benchmark], &output_path).expect("write results");
    println!("Results written to: {}", output_path.display());
}

/// Extra-large scale synthetic benchmark (250k issues).
/// Env gate: BR_E2E_STRESS=1
#[test]
#[ignore = "stress test: BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale -- --ignored"]
fn stress_synthetic_xlarge() {
    if !stress_tests_enabled() {
        eprintln!("Skipping stress test (set BR_E2E_STRESS=1 to enable)");
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");

    println!("\n=== Synthetic Scale-Up Benchmark: XLarge (250K) ===\n");

    let config = SyntheticConfig::from_tier(ScaleTier::XLarge);
    eprintln!(
        "Generating synthetic dataset ({} issues)...",
        config.issue_count
    );

    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("Failed to generate synthetic dataset");

    eprintln!("Running benchmarks...");
    let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    print_benchmark(&benchmark);

    // Write results
    let output_dir = PathBuf::from("target/benchmark-results");
    fs::create_dir_all(&output_dir).expect("create output dir");
    let output_path = output_dir.join("synthetic_xlarge_latest.json");
    write_benchmark_json(&[benchmark], &output_path).expect("write results");
    println!("Results written to: {}", output_path.display());
}

/// Manual million-issue synthetic benchmark with 10,000 simulated agents.
/// Env gate: BR_E2E_STRESS=1 BR_SYNTHETIC_MILLION=1
#[test]
#[ignore = "manual stress test: BR_E2E_STRESS=1 BR_SYNTHETIC_MILLION=1 cargo test --test bench_synthetic_scale stress_synthetic_million -- --ignored --nocapture"]
fn stress_synthetic_million() {
    if !stress_tests_enabled() || !million_profile_enabled() {
        eprintln!(
            "Skipping million-issue stress test (set BR_E2E_STRESS=1 BR_SYNTHETIC_MILLION=1)"
        );
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");
    let seed = synthetic_seed_from_env(42);

    println!("\n=== Synthetic Scale-Up Benchmark: Million (1M / 10K agents) ===\n");

    let config = SyntheticConfig::million_agent_profile(seed);
    eprintln!(
        "Generating synthetic dataset ({} issues, {} simulated agents)...",
        config.issue_count, config.simulated_agent_count
    );

    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("Failed to generate synthetic million-agent dataset");

    eprintln!("Running benchmarks...");
    let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    print_benchmark(&benchmark);

    let output_dir = PathBuf::from("target/benchmark-results");
    fs::create_dir_all(&output_dir).expect("create output dir");
    let output_path = output_dir.join("synthetic_million_latest.json");
    write_benchmark_json(&[benchmark], &output_path).expect("write results");
    println!("Results written to: {}", output_path.display());
    println!("Manifest written to: {}", dataset.manifest_path.display());
}

/// Run all synthetic benchmarks in sequence.
/// Env gate: BR_E2E_STRESS=1
#[test]
#[ignore = "stress test: BR_E2E_STRESS=1 cargo test --test bench_synthetic_scale -- --ignored"]
fn stress_synthetic_all() {
    if !stress_tests_enabled() {
        eprintln!("Skipping stress test (set BR_E2E_STRESS=1 to enable)");
        return;
    }

    let binaries = discover_binaries().expect("Binary discovery failed");
    let mut all_benchmarks = Vec::new();

    println!("\n=== Synthetic Scale-Up Benchmark Suite ===\n");

    for tier in [ScaleTier::Small, ScaleTier::Medium, ScaleTier::Large] {
        let config = SyntheticConfig::from_tier(tier);
        eprintln!(
            "\n[{}] Generating {} issues...",
            tier.name(),
            config.issue_count
        );

        match SyntheticDataset::generate(config, &binaries.br.path) {
            Ok(dataset) => {
                eprintln!("[{}] Running benchmarks...", tier.name());
                let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
                print_benchmark(&benchmark);
                all_benchmarks.push(benchmark);
            }
            Err(e) => {
                eprintln!("[{}] FAILED: {e}", tier.name());
            }
        }
    }

    // Write combined results
    if !all_benchmarks.is_empty() {
        let output_dir = PathBuf::from("target/benchmark-results");
        fs::create_dir_all(&output_dir).expect("create output dir");

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("synthetic_all_{timestamp}.json"));
        write_benchmark_json(&all_benchmarks, &output_path).expect("write results");
        println!("\nAll results written to: {}", output_path.display());

        // Also write latest.json
        let latest_path = output_dir.join("synthetic_all_latest.json");
        write_benchmark_json(&all_benchmarks, &latest_path).expect("write latest");
    }

    // Print overall summary
    println!("\n{}", "=".repeat(80));
    println!("OVERALL SUMMARY");
    println!("{}", "=".repeat(80));

    for b in &all_benchmarks {
        let ips = b
            .summary
            .issues_per_second
            .map_or_else(|| "N/A".to_string(), |v| format!("{:.0}", v));
        println!(
            "{}: {}ms total, {} issues/sec for list",
            b.tier, b.summary.total_duration_ms, ips
        );
    }
}

/// Unit test for synthetic config creation.
#[test]
fn test_synthetic_config_from_tier() {
    let config = SyntheticConfig::from_tier(ScaleTier::Large);
    assert_eq!(config.issue_count, 100_000);
    assert!((config.dependency_density - 0.5).abs() < 0.01);
    assert_eq!(config.seed, 42);
}

/// Unit test for scale tier properties.
#[test]
fn test_scale_tier_properties() {
    assert_eq!(ScaleTier::Small.issue_count(), 10_000);
    assert_eq!(ScaleTier::Medium.issue_count(), 50_000);
    assert_eq!(ScaleTier::Large.issue_count(), 100_000);
    assert_eq!(ScaleTier::XLarge.issue_count(), 250_000);
    assert_eq!(ScaleTier::Million.issue_count(), 1_000_000);

    assert_eq!(ScaleTier::Small.name(), "small_10k");
    assert_eq!(ScaleTier::Large.name(), "large_100k");
    assert_eq!(ScaleTier::Million.name(), "million_1m");
}

/// Unit test for title generation.
#[test]
fn test_generate_title() {
    let mut rng = StdRng::seed_from_u64(42);
    let title = generate_title(&mut rng, 123);

    // Should have format "Prefix subject (#123)"
    assert!(title.contains("#123"));
    assert!(title.len() > 10);
}

#[test]
fn test_synthetic_config_distribution_builders() {
    let config = SyntheticConfig::ci_profile(7)
        .with_issue_count(64)
        .with_label_distribution(8, 1, 2)
        .with_comment_distribution(0.5, 3)
        .with_agent_distribution(32, 0.25)
        .with_dag_skew(2.0);

    assert_eq!(config.issue_count, 64);
    assert_eq!(config.seed, 7);
    assert_eq!(config.label_pool_size, 8);
    assert_eq!(config.min_labels_per_issue, 1);
    assert_eq!(config.max_labels_per_issue, 2);
    assert!((config.comment_density - 0.5).abs() < f64::EPSILON);
    assert_eq!(config.max_comments_per_issue, 3);
    assert_eq!(config.simulated_agent_count, 32);
    assert!((config.claim_density - 0.25).abs() < f64::EPSILON);
    assert!((config.dag_skew - 2.0).abs() < f64::EPSILON);
}

#[test]
fn synthetic_graph_workloads_cover_skewed_and_wide_profiles() {
    let workloads = synthetic_graph_workloads(96);
    let names = workloads
        .iter()
        .map(|workload| workload.operation)
        .collect::<BTreeSet<_>>();

    assert!(names.contains("graph_hot_hub"));
    assert!(names.contains("dep_tree_hot_leaf"));
    assert!(names.contains("graph_all_components"));

    let dep_tree = workloads
        .iter()
        .find(|workload| workload.operation == "dep_tree_hot_leaf")
        .expect("leaf dependency-tree workload should exist");
    assert!(dep_tree.args.windows(2).any(|window| {
        window.first().is_some_and(|arg| arg == "--max-depth")
            && window.get(1).is_some_and(|arg| arg == "12")
    }));

    let million_workloads = synthetic_graph_workloads(ScaleTier::Million.issue_count());
    assert!(
        million_workloads
            .iter()
            .any(|workload| workload.operation == "graph_hot_hub")
    );
    assert!(
        million_workloads
            .iter()
            .any(|workload| workload.operation == "dep_tree_hot_leaf")
    );
    assert!(
        million_workloads
            .iter()
            .all(|workload| workload.operation != "graph_all_components")
    );
}

#[test]
fn parse_time_max_rss_bytes_reads_gnu_time_output() {
    let stderr = "\
Command being timed: \"br list --json\"\n\
\tUser time (seconds): 0.03\n\
\tMaximum resident set size (kbytes): 12345\n\
\tExit status: 0\n";

    assert_eq!(parse_time_max_rss_bytes(stderr), Some(12_641_280));
}

#[test]
fn sha256_hex_hashes_stdout_for_evidence() {
    assert_eq!(
        sha256_hex(b"br evidence\n"),
        "8dfb2f8fd989532fa7371e6787180e687f541d91995e9a8c27378bc3afbd5406"
    );
}

#[test]
fn synthetic_ci_profile_benchmarks_graph_projection_workloads() {
    let binaries = discover_binaries().expect("Binary discovery failed");
    let config = SyntheticConfig::ci_profile(101)
        .with_issue_count(96)
        .with_dag_skew(2.0);
    let dataset = SyntheticDataset::generate(config, &binaries.br.path)
        .expect("generate CI synthetic corpus");

    let benchmark = benchmark_synthetic(&dataset, &binaries.br.path);
    let failed_operations = benchmark
        .operations
        .iter()
        .filter(|operation| !operation.success)
        .map(|operation| operation.operation.as_str())
        .collect::<Vec<_>>();
    assert!(
        failed_operations.is_empty(),
        "synthetic CI profile should have no failed operations: {failed_operations:?}"
    );
    assert_eq!(benchmark.config.issue_count, 96);
    assert_eq!(
        benchmark.br_binary_path,
        binaries.br.path.display().to_string()
    );
    assert!(
        benchmark
            .reproduction_command
            .contains("BR_SYNTHETIC_SEED=101")
    );
    let operation_names = benchmark
        .operations
        .iter()
        .map(|operation| operation.operation.as_str())
        .collect::<BTreeSet<_>>();

    for expected in ["graph_hot_hub", "dep_tree_hot_leaf", "graph_all_components"] {
        assert!(operation_names.contains(expected), "missing {expected}");
        let operation = benchmark
            .operations
            .iter()
            .find(|operation| operation.operation == expected)
            .expect("expected operation should be recorded");
        assert!(
            operation.success,
            "{expected} failed: {:?}",
            operation.error.as_deref()
        );
        assert!(
            operation.output_size_bytes > 0,
            "{expected} should emit measurable output"
        );
        assert_eq!(
            operation.stdout_sha256.as_deref().map(str::len),
            Some(64),
            "{expected} should include stdout hash evidence"
        );
    }

    if Path::new("/usr/bin/time").is_file() {
        let missing_rss = benchmark
            .operations
            .iter()
            .filter(|operation| operation.success && operation.peak_rss_bytes.is_none())
            .map(|operation| operation.operation.as_str())
            .collect::<Vec<_>>();
        assert!(
            missing_rss.is_empty(),
            "GNU time should provide child-process RSS for every successful operation; missing: {missing_rss:?}"
        );
    }
}

#[test]
fn synthetic_ci_profile_generates_valid_reproducible_manifest() {
    let binaries = discover_binaries().expect("Binary discovery failed");
    let config = SyntheticConfig::ci_profile(99).with_issue_count(96);

    let first = SyntheticDataset::generate(config.clone(), &binaries.br.path)
        .expect("generate first CI synthetic corpus");
    let second =
        SyntheticDataset::generate(config, &binaries.br.path).expect("generate second CI corpus");

    assert_eq!(first.metrics.issue_count, 96);
    assert_eq!(first.metrics.health.jsonl_issue_count, 96);
    assert_eq!(first.metrics.content_hash, second.metrics.content_hash);
    assert_eq!(
        first.metrics.expected_jsonl_size_bytes,
        first.metrics.jsonl_size_bytes
    );
    assert!(first.metrics.health.jsonl_valid);
    assert!(first.metrics.health.sync_import_ok);
    assert!(first.metrics.health.doctor_ok);
    assert!(first.metrics.health.sync_status_clean);
    assert!(first.metrics.dependency_count > 0);
    assert!(first.metrics.label_assignment_count > 0);
    assert!(first.metrics.comment_count > 0);
    assert!(first.metrics.claim_count > 0);
    assert_eq!(first.metrics.simulated_agent_count, 16);

    let manifest = fs::read_to_string(&first.manifest_path).expect("read corpus manifest");
    let manifest: SyntheticCorpusManifest =
        serde_json::from_str(&manifest).expect("parse corpus manifest");
    assert_eq!(manifest.schema_version, "br.synthetic-corpus.v1");
    assert_eq!(manifest.config.seed, 99);
    assert_eq!(manifest.metrics.content_hash, first.metrics.content_hash);
    assert!(
        manifest
            .reproduction_command
            .contains("BR_SYNTHETIC_SEED=99")
    );
}
