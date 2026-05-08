//! Pure evidence contracts for swarm coordination diagnosis.
//!
//! This module deliberately does not inspect Agent Mail, read the filesystem, or
//! mutate claims. Callers provide issue metadata and optional reservation
//! evidence, then receive a deterministic classification that future CLI, MCP,
//! scheduler, and audit surfaces can share.

use crate::model::{Comment, IssueType, Priority, Status};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Schema version for coordination status and claim evidence outputs.
pub const COORDINATION_SCHEMA_VERSION: &str = "br.coordination.v1";
/// Swarm-agent claims become stale candidates after two quiet hours.
pub const SWARM_STALE_CANDIDATE_AFTER_MINUTES: i64 = 2 * 60;
/// Extra-conservative marker for likely abandoned swarm claims.
pub const SWARM_ABANDONED_LIKELY_AFTER_MINUTES: i64 = 8 * 60;
/// Human or unclear claims use a one-business-day rule of thumb.
pub const HUMAN_STALE_CANDIDATE_AFTER_MINUTES: i64 = 24 * 60;
/// Extra-conservative marker for likely abandoned human or unclear claims.
pub const HUMAN_ABANDONED_LIKELY_AFTER_MINUTES: i64 = 72 * 60;

/// Who appears to own the current claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimOwnerKind {
    /// A named coding-agent swarm participant.
    SwarmAgent,
    /// A human assignee or operator-owned claim.
    Human,
    /// Ownership cannot be confidently classified.
    Unknown,
}

impl ClaimOwnerKind {
    /// Stale-candidate threshold in minutes for this owner class.
    #[must_use]
    pub const fn stale_candidate_after_minutes(self) -> i64 {
        match self {
            Self::SwarmAgent => SWARM_STALE_CANDIDATE_AFTER_MINUTES,
            Self::Human | Self::Unknown => HUMAN_STALE_CANDIDATE_AFTER_MINUTES,
        }
    }

    /// Likely-abandoned threshold in minutes for this owner class.
    #[must_use]
    pub const fn abandoned_likely_after_minutes(self) -> i64 {
        match self {
            Self::SwarmAgent => SWARM_ABANDONED_LIKELY_AFTER_MINUTES,
            Self::Human | Self::Unknown => HUMAN_ABANDONED_LIKELY_AFTER_MINUTES,
        }
    }
}

/// Optional Agent Mail reservation evidence supplied by the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "state", content = "detail")]
pub enum ReservationEvidence {
    /// No Agent Mail snapshot was supplied, so absence of reservations is not
    /// evidence of abandonment.
    NoSnapshot,
    /// A snapshot was supplied and no matching reservation was found.
    NoReservation,
    /// A matching active reservation exists.
    Active {
        holder: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<DateTime<Utc>>,
    },
    /// A matching reservation exists but is no longer active.
    Expired {
        holder: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        released_at: Option<DateTime<Utc>>,
    },
    /// Snapshot data was supplied but could not be trusted.
    InvalidSnapshot { reason: String },
}

/// Stable claim classifications used by coordination-aware surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimClassification {
    /// The issue is in progress but has no meaningful assignee.
    Unassigned,
    /// The claim is still within the owner-specific freshness window.
    Fresh,
    /// A live reservation exists, so the claim must not be treated as abandoned.
    BlockedByActiveReservation,
    /// The claim has crossed the stale threshold and no active reservation was
    /// found in a supplied snapshot.
    StaleCandidate,
    /// The claim has crossed a more conservative abandoned threshold.
    AbandonedLikely,
    /// The claim is old enough to inspect, but no Agent Mail snapshot was
    /// supplied.
    NoMailSnapshot,
    /// Evidence conflicts or cannot be trusted.
    Ambiguous,
}

impl ClaimClassification {
    /// Stable snake_case value used in text projections.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unassigned => "unassigned",
            Self::Fresh => "fresh",
            Self::BlockedByActiveReservation => "blocked_by_active_reservation",
            Self::StaleCandidate => "stale_candidate",
            Self::AbandonedLikely => "abandoned_likely",
            Self::NoMailSnapshot => "no_mail_snapshot",
            Self::Ambiguous => "ambiguous",
        }
    }
}

/// Suggested next action for an operator or agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedAction {
    /// Do nothing except continue normal observation.
    Observe,
    /// Ask the assignee or human operator before touching the claim.
    AskOwner,
    /// Inspect Agent Mail reservations or capture a fresh snapshot.
    InspectMail,
    /// The issue is a candidate for the documented audit-comment plus claim
    /// sequence. This is still advisory, not an automatic mutation.
    ReclaimCandidate,
    /// Leave the claim alone because evidence says work may still be active.
    LeaveActive,
}

impl RecommendedAction {
    /// Stable snake_case value used in text projections.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::AskOwner => "ask_owner",
            Self::InspectMail => "inspect_mail",
            Self::ReclaimCandidate => "reclaim_candidate",
            Self::LeaveActive => "leave_active",
        }
    }
}

/// Evidence categories present in a claim assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationEvidenceSource {
    /// Issue status, assignee, and updated timestamp.
    IssueMetadata,
    /// Owner-specific stale and abandoned thresholds.
    PolicyThreshold,
    /// A caller-supplied Agent Mail snapshot.
    AgentMailSnapshot,
    /// Explicit absence of an Agent Mail snapshot.
    NoAgentMailSnapshot,
}

/// Bounded comment excerpt included with a coordination claim row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationComment {
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Comment> for CoordinationComment {
    fn from(comment: &Comment) -> Self {
        Self {
            author: comment.author.clone(),
            text: comment.body.clone(),
            created_at: comment.created_at,
        }
    }
}

/// Caller-provided input for one claim assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimAssessmentInput {
    pub assignee: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
    pub owner_kind: ClaimOwnerKind,
    pub reservation: ReservationEvidence,
}

/// Deterministic assessment for one in-progress claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClaimAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    pub owner_kind: ClaimOwnerKind,
    pub updated_at: DateTime<Utc>,
    pub updated_age_minutes: i64,
    pub stale_threshold_minutes: i64,
    pub abandoned_threshold_minutes: i64,
    pub reservation: ReservationEvidence,
    pub classification: ClaimClassification,
    pub recommended_action: RecommendedAction,
    pub evidence_sources: Vec<CoordinationEvidenceSource>,
}

/// Issue context attached to one coordination claim row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationIssueRow {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub issue_type: IssueType,
    pub labels: Vec<String>,
    pub dependency_count: usize,
    pub dependent_count: usize,
    pub latest_comments: Vec<CoordinationComment>,
}

/// One in-progress issue plus its deterministic coordination assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationClaimRow {
    pub issue: CoordinationIssueRow,
    pub assessment: ClaimAssessment,
}

/// Workspace context counts shown next to hidden-claim diagnosis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationWorkspaceCounts {
    pub open: usize,
    pub ready: usize,
    pub blocked: usize,
    pub in_progress: usize,
    pub deferred: usize,
    pub closed: usize,
}

/// Count summary for coordination status output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationSummary {
    pub total_claims: usize,
    pub workspace: CoordinationWorkspaceCounts,
    pub unassigned: usize,
    pub fresh: usize,
    pub blocked_by_active_reservation: usize,
    pub stale_candidate: usize,
    pub abandoned_likely: usize,
    pub no_mail_snapshot: usize,
    pub ambiguous: usize,
}

/// Top-level machine-readable coordination status shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationStatusOutput {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub summary: CoordinationSummary,
    pub claims: Vec<CoordinationClaimRow>,
}

impl CoordinationStatusOutput {
    /// Build a coordination status envelope with current schema version.
    #[must_use]
    pub fn new(
        generated_at: DateTime<Utc>,
        workspace: CoordinationWorkspaceCounts,
        claims: Vec<CoordinationClaimRow>,
    ) -> Self {
        let summary = CoordinationSummary::from_claims(&claims, workspace);
        Self {
            schema_version: COORDINATION_SCHEMA_VERSION.to_string(),
            generated_at,
            summary,
            claims,
        }
    }
}

impl CoordinationSummary {
    /// Count claim classifications for a status envelope.
    #[must_use]
    pub fn from_claims(
        claims: &[CoordinationClaimRow],
        workspace: CoordinationWorkspaceCounts,
    ) -> Self {
        let mut summary = Self {
            total_claims: claims.len(),
            workspace,
            unassigned: 0,
            fresh: 0,
            blocked_by_active_reservation: 0,
            stale_candidate: 0,
            abandoned_likely: 0,
            no_mail_snapshot: 0,
            ambiguous: 0,
        };

        for claim in claims {
            match claim.assessment.classification {
                ClaimClassification::Unassigned => summary.unassigned += 1,
                ClaimClassification::Fresh => summary.fresh += 1,
                ClaimClassification::BlockedByActiveReservation => {
                    summary.blocked_by_active_reservation += 1;
                }
                ClaimClassification::StaleCandidate => summary.stale_candidate += 1,
                ClaimClassification::AbandonedLikely => summary.abandoned_likely += 1,
                ClaimClassification::NoMailSnapshot => summary.no_mail_snapshot += 1,
                ClaimClassification::Ambiguous => summary.ambiguous += 1,
            }
        }

        summary
    }
}

/// Classify one in-progress claim from caller-supplied evidence.
#[must_use]
pub fn assess_claim(input: ClaimAssessmentInput) -> ClaimAssessment {
    let assignee = input
        .assignee
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let updated_age_minutes = input
        .now
        .signed_duration_since(input.updated_at)
        .num_minutes()
        .max(0);
    let stale_threshold_minutes = input.owner_kind.stale_candidate_after_minutes();
    let abandoned_threshold_minutes = input.owner_kind.abandoned_likely_after_minutes();

    let (classification, recommended_action) = classify_claim(
        assignee.as_deref(),
        input.owner_kind,
        &input.reservation,
        updated_age_minutes,
        stale_threshold_minutes,
        abandoned_threshold_minutes,
    );
    let evidence_sources = evidence_sources_for(&input.reservation);

    ClaimAssessment {
        assignee,
        owner_kind: input.owner_kind,
        updated_at: input.updated_at,
        updated_age_minutes,
        stale_threshold_minutes,
        abandoned_threshold_minutes,
        reservation: input.reservation,
        classification,
        recommended_action,
        evidence_sources,
    }
}

fn classify_claim(
    assignee: Option<&str>,
    owner_kind: ClaimOwnerKind,
    reservation: &ReservationEvidence,
    updated_age_minutes: i64,
    stale_threshold_minutes: i64,
    abandoned_threshold_minutes: i64,
) -> (ClaimClassification, RecommendedAction) {
    if assignee.is_none() {
        return (ClaimClassification::Unassigned, RecommendedAction::Observe);
    }

    if updated_age_minutes < stale_threshold_minutes {
        return (ClaimClassification::Fresh, RecommendedAction::Observe);
    }

    match reservation {
        ReservationEvidence::Active { .. } => (
            ClaimClassification::BlockedByActiveReservation,
            RecommendedAction::LeaveActive,
        ),
        ReservationEvidence::NoSnapshot => (
            ClaimClassification::NoMailSnapshot,
            RecommendedAction::InspectMail,
        ),
        ReservationEvidence::InvalidSnapshot { .. } => (
            ClaimClassification::Ambiguous,
            RecommendedAction::InspectMail,
        ),
        ReservationEvidence::NoReservation | ReservationEvidence::Expired { .. } => {
            if updated_age_minutes >= abandoned_threshold_minutes {
                (
                    ClaimClassification::AbandonedLikely,
                    recommended_reclaim_action(owner_kind),
                )
            } else {
                (
                    ClaimClassification::StaleCandidate,
                    recommended_reclaim_action(owner_kind),
                )
            }
        }
    }
}

const fn recommended_reclaim_action(owner_kind: ClaimOwnerKind) -> RecommendedAction {
    match owner_kind {
        ClaimOwnerKind::SwarmAgent => RecommendedAction::ReclaimCandidate,
        ClaimOwnerKind::Human | ClaimOwnerKind::Unknown => RecommendedAction::AskOwner,
    }
}

fn evidence_sources_for(reservation: &ReservationEvidence) -> Vec<CoordinationEvidenceSource> {
    let mut sources = vec![
        CoordinationEvidenceSource::IssueMetadata,
        CoordinationEvidenceSource::PolicyThreshold,
    ];
    match reservation {
        ReservationEvidence::NoSnapshot => {
            sources.push(CoordinationEvidenceSource::NoAgentMailSnapshot);
        }
        ReservationEvidence::NoReservation
        | ReservationEvidence::Active { .. }
        | ReservationEvidence::Expired { .. }
        | ReservationEvidence::InvalidSnapshot { .. } => {
            sources.push(CoordinationEvidenceSource::AgentMailSnapshot);
        }
    }
    sources
}

#[cfg(test)]
mod tests {
    use super::{
        COORDINATION_SCHEMA_VERSION, ClaimAssessmentInput, ClaimClassification, ClaimOwnerKind,
        CoordinationClaimRow, CoordinationComment, CoordinationEvidenceSource,
        CoordinationIssueRow, CoordinationStatusOutput, CoordinationWorkspaceCounts,
        RecommendedAction, ReservationEvidence, assess_claim,
    };
    use crate::model::{IssueType, Priority, Status};
    use chrono::{Duration, TimeZone, Utc};

    fn now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 8, 9, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn input(
        minutes_old: i64,
        owner_kind: ClaimOwnerKind,
        reservation: ReservationEvidence,
    ) -> ClaimAssessmentInput {
        ClaimAssessmentInput {
            assignee: Some("TopazFox".to_string()),
            updated_at: now() - Duration::minutes(minutes_old),
            now: now(),
            owner_kind,
            reservation,
        }
    }

    fn claim_row(id: &str, assessment: super::ClaimAssessment) -> CoordinationClaimRow {
        CoordinationClaimRow {
            issue: CoordinationIssueRow {
                id: id.to_string(),
                title: format!("Claim {id}"),
                status: Status::InProgress,
                priority: Priority(1),
                issue_type: IssueType::Task,
                labels: vec!["coordination".to_string()],
                dependency_count: 1,
                dependent_count: 2,
                latest_comments: vec![CoordinationComment {
                    author: "TopazFox".to_string(),
                    text: "latest evidence".to_string(),
                    created_at: now(),
                }],
            },
            assessment,
        }
    }

    #[test]
    fn swarm_claim_below_two_hours_is_fresh() {
        let assessment = assess_claim(input(
            119,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::NoReservation,
        ));

        assert_eq!(assessment.updated_age_minutes, 119);
        assert_eq!(assessment.stale_threshold_minutes, 120);
        assert_eq!(assessment.classification, ClaimClassification::Fresh);
        assert_eq!(assessment.recommended_action, RecommendedAction::Observe);
    }

    #[test]
    fn old_swarm_claim_without_mail_snapshot_requires_mail_inspection() {
        let assessment = assess_claim(input(
            120,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::NoSnapshot,
        ));

        assert_eq!(
            assessment.classification,
            ClaimClassification::NoMailSnapshot
        );
        assert_eq!(
            assessment.recommended_action,
            RecommendedAction::InspectMail
        );
        assert!(
            assessment
                .evidence_sources
                .contains(&CoordinationEvidenceSource::NoAgentMailSnapshot)
        );
    }

    #[test]
    fn absent_reservation_snapshot_match_marks_stale_candidate() {
        let assessment = assess_claim(input(
            120,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::NoReservation,
        ));

        assert_eq!(
            assessment.classification,
            ClaimClassification::StaleCandidate
        );
        assert_eq!(
            assessment.recommended_action,
            RecommendedAction::ReclaimCandidate
        );
    }

    #[test]
    fn very_old_swarm_claim_is_abandoned_likely() {
        let assessment = assess_claim(input(
            8 * 60,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::Expired {
                holder: "TopazFox".to_string(),
                released_at: None,
            },
        ));

        assert_eq!(
            assessment.classification,
            ClaimClassification::AbandonedLikely
        );
        assert_eq!(assessment.abandoned_threshold_minutes, 8 * 60);
        assert_eq!(
            assessment.recommended_action,
            RecommendedAction::ReclaimCandidate
        );
    }

    #[test]
    fn active_reservation_blocks_reclaim_even_when_old() {
        let assessment = assess_claim(input(
            12 * 60,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::Active {
                holder: "TopazFox".to_string(),
                expires_at: Some(now() + Duration::minutes(30)),
            },
        ));

        assert_eq!(
            assessment.classification,
            ClaimClassification::BlockedByActiveReservation
        );
        assert_eq!(
            assessment.recommended_action,
            RecommendedAction::LeaveActive
        );
    }

    #[test]
    fn human_and_unknown_claims_use_one_business_day_threshold() {
        let human_fresh = assess_claim(input(
            23 * 60,
            ClaimOwnerKind::Human,
            ReservationEvidence::NoReservation,
        ));
        let unknown_stale = assess_claim(input(
            24 * 60,
            ClaimOwnerKind::Unknown,
            ReservationEvidence::NoReservation,
        ));

        assert_eq!(human_fresh.classification, ClaimClassification::Fresh);
        assert_eq!(unknown_stale.stale_threshold_minutes, 24 * 60);
        assert_eq!(
            unknown_stale.classification,
            ClaimClassification::StaleCandidate
        );
        assert_eq!(
            unknown_stale.recommended_action,
            RecommendedAction::AskOwner
        );
    }

    #[test]
    fn future_updated_at_saturates_age_to_zero() {
        let assessment = assess_claim(ClaimAssessmentInput {
            assignee: Some("TopazFox".to_string()),
            updated_at: now() + Duration::minutes(5),
            now: now(),
            owner_kind: ClaimOwnerKind::SwarmAgent,
            reservation: ReservationEvidence::NoReservation,
        });

        assert_eq!(assessment.updated_age_minutes, 0);
        assert_eq!(assessment.classification, ClaimClassification::Fresh);
    }

    #[test]
    fn empty_or_whitespace_assignee_is_unassigned() {
        let assessment = assess_claim(ClaimAssessmentInput {
            assignee: Some("   ".to_string()),
            updated_at: now() - Duration::hours(12),
            now: now(),
            owner_kind: ClaimOwnerKind::SwarmAgent,
            reservation: ReservationEvidence::NoReservation,
        });

        assert_eq!(assessment.assignee, None);
        assert_eq!(assessment.classification, ClaimClassification::Unassigned);
        assert_eq!(assessment.recommended_action, RecommendedAction::Observe);
    }

    #[test]
    fn invalid_snapshot_is_ambiguous_not_abandoned() {
        let assessment = assess_claim(input(
            8 * 60,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::InvalidSnapshot {
                reason: "missing holder field".to_string(),
            },
        ));

        assert_eq!(assessment.classification, ClaimClassification::Ambiguous);
        assert_eq!(
            assessment.recommended_action,
            RecommendedAction::InspectMail
        );
    }

    #[test]
    fn status_output_sets_schema_and_counts_classifications() {
        let fresh = assess_claim(input(
            30,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::NoReservation,
        ));
        let stale = assess_claim(input(
            120,
            ClaimOwnerKind::SwarmAgent,
            ReservationEvidence::NoReservation,
        ));
        let workspace = CoordinationWorkspaceCounts {
            open: 3,
            ready: 1,
            blocked: 1,
            in_progress: 2,
            deferred: 0,
            closed: 5,
        };
        let output = CoordinationStatusOutput::new(
            now(),
            workspace,
            vec![claim_row("bd-fresh", fresh), claim_row("bd-stale", stale)],
        );

        assert_eq!(output.schema_version, COORDINATION_SCHEMA_VERSION);
        assert_eq!(output.summary.total_claims, 2);
        assert_eq!(output.summary.workspace.in_progress, 2);
        assert_eq!(output.summary.fresh, 1);
        assert_eq!(output.summary.stale_candidate, 1);
        assert_eq!(
            output.claims[0].issue.latest_comments[0].text,
            "latest evidence"
        );
    }

    #[test]
    fn coordination_status_schema_declares_required_sections() {
        let schema = schemars::schema_for!(CoordinationStatusOutput);
        let value = serde_json::to_value(schema).expect("schema serializes");
        let required = value["required"].as_array().expect("schema has required");
        for field in ["schema_version", "generated_at", "summary", "claims"] {
            assert!(
                required.iter().any(|entry| entry.as_str() == Some(field)),
                "CoordinationStatusOutput schema should require {field}"
            );
        }
    }
}
