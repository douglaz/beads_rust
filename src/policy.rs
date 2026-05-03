//! Deterministic adaptive policy documents.
//!
//! This module defines the data contract used by future swarm-scale features
//! when runtime behavior needs to be chosen from recorded evidence. Evaluation
//! is pure: callers provide every input and evidence value, and the evaluator
//! only compares those values against the policy document.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Current policy document schema version.
pub const POLICY_SCHEMA_VERSION: u32 = 1;

/// A complete adaptive policy document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdaptivePolicy {
    /// Schema version of this policy document format.
    pub schema_version: u32,
    /// Compatibility boundary for decisions made from this policy.
    pub compatibility_version: CompatibilityVersion,
    /// Stable identifier for audit logs and replay bundles.
    pub policy_id: String,
    /// Caller-provided operation inputs.
    pub inputs: Vec<PolicyInput>,
    /// Outputs this policy may decide.
    pub outputs: Vec<PolicyOutput>,
    /// Resource or latency budgets the policy promises to respect.
    pub budgets: Vec<PolicyBudget>,
    /// Evidence fields read by the evaluator.
    pub evidence_fields: Vec<EvidenceField>,
    /// Conservative decision returned when evidence is missing or invalid.
    pub fallback: PolicyFallback,
    /// Ordered rules. The first matching rule wins.
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

impl AdaptivePolicy {
    /// Validate internal references, required fallback coverage, and value
    /// types without reading any external state.
    pub fn validate(&self) -> Result<(), Vec<PolicyValidationError>> {
        let mut errors = Vec::new();

        if self.schema_version != POLICY_SCHEMA_VERSION {
            errors.push(PolicyValidationError::new(
                "schema_version",
                format!(
                    "unsupported schema version {}; expected {POLICY_SCHEMA_VERSION}",
                    self.schema_version
                ),
            ));
        }
        validate_name("policy_id", &self.policy_id, &mut errors);

        let inputs = validate_inputs(&self.inputs, &mut errors);
        let outputs = validate_outputs(&self.outputs, &mut errors);
        let evidence_fields = validate_evidence_fields(&self.evidence_fields, &mut errors);
        validate_budgets(&self.budgets, &mut errors);

        validate_output_values(
            "fallback.outputs",
            &self.fallback.outputs,
            &outputs,
            true,
            &mut errors,
        );

        for (index, rule) in self.rules.iter().enumerate() {
            let rule_scope = format!("rules[{index}]");
            validate_name(&format!("{rule_scope}.rule_id"), &rule.rule_id, &mut errors);
            validate_output_values(
                &format!("{rule_scope}.outputs"),
                &rule.outputs,
                &outputs,
                false,
                &mut errors,
            );
            validate_rule_conditions(rule, index, &inputs, &evidence_fields, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Evaluate this policy against caller-provided context.
    ///
    /// Missing or invalid required evidence fails closed to `fallback`.
    #[must_use]
    pub fn evaluate(&self, context: &PolicyContext) -> PolicyDecision {
        if let Err(errors) = self.validate() {
            return self.fallback_decision(format!(
                "invalid_policy: {}",
                summarize_validation_errors(&errors)
            ));
        }

        if let Some(reason) = self.context_failure(context) {
            return self.fallback_decision(reason);
        }

        for rule in &self.rules {
            match rule.matches(context) {
                Ok(true) => return self.rule_decision(rule),
                Ok(false) => {}
                Err(reason) => return self.fallback_decision(reason),
            }
        }

        self.fallback_decision("no_rule_matched".to_string())
    }

    fn context_failure(&self, context: &PolicyContext) -> Option<String> {
        for input in &self.inputs {
            if let Some(value) = context.inputs.get(&input.name) {
                if !input.value_type.accepts(value) {
                    return Some(format!(
                        "invalid_input: {} expected {} got {}",
                        input.name,
                        input.value_type.as_str(),
                        value.value_type().as_str()
                    ));
                }
            } else if input.required {
                return Some(format!("missing_required_input: {}", input.name));
            }
        }

        for evidence in &self.evidence_fields {
            if let Some(value) = context.evidence.get(&evidence.name) {
                if !evidence.value_type.accepts(value) {
                    return Some(format!(
                        "invalid_evidence: {} expected {} got {}",
                        evidence.name,
                        evidence.value_type.as_str(),
                        value.value_type().as_str()
                    ));
                }
            } else if evidence.required {
                return Some(format!("missing_required_evidence: {}", evidence.name));
            }
        }

        None
    }

    fn fallback_decision(&self, reason: String) -> PolicyDecision {
        PolicyDecision {
            policy_id: self.policy_id.clone(),
            compatibility_version: self.compatibility_version.clone(),
            outputs: self.fallback.outputs.clone(),
            fallback_active: true,
            applied_rule: None,
            reason,
        }
    }

    fn rule_decision(&self, rule: &PolicyRule) -> PolicyDecision {
        let mut outputs = self.fallback.outputs.clone();
        outputs.extend(rule.outputs.clone());

        PolicyDecision {
            policy_id: self.policy_id.clone(),
            compatibility_version: self.compatibility_version.clone(),
            outputs,
            fallback_active: false,
            applied_rule: Some(rule.rule_id.clone()),
            reason: rule.reason.clone(),
        }
    }
}

/// Compatibility boundary for a policy family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityVersion {
    /// Human-readable family, such as `swarm_scale`.
    pub family: String,
    /// Monotonic major version. Consumers must not silently cross this line.
    pub major: u32,
    /// Monotonic minor version for additive compatible changes.
    pub minor: u32,
}

/// A declared operation input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyInput {
    pub name: String,
    pub value_type: PolicyValueType,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A declared decision output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyOutput {
    pub name: String,
    pub value_type: PolicyValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A declared evidence field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceField {
    pub name: String,
    pub value_type: PolicyValueType,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Resource or latency budget metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyBudget {
    pub name: String,
    pub value: u64,
    pub unit: BudgetUnit,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Units available for policy budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BudgetUnit {
    Bytes,
    Milliseconds,
    Percent,
    Count,
}

/// Conservative output set used when the evaluator cannot trust evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyFallback {
    pub outputs: BTreeMap<String, PolicyValue>,
    pub reason: String,
}

/// A deterministic first-match rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    pub rule_id: String,
    #[serde(default)]
    pub when_all: Vec<PolicyCondition>,
    pub outputs: BTreeMap<String, PolicyValue>,
    pub reason: String,
}

impl PolicyRule {
    fn matches(&self, context: &PolicyContext) -> Result<bool, String> {
        for condition in &self.when_all {
            if !condition.matches(context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// One predicate over an input or evidence field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyCondition {
    pub source: PolicyValueSource,
    pub field: String,
    pub operator: PolicyOperator,
    pub value: PolicyValue,
}

impl PolicyCondition {
    fn matches(&self, context: &PolicyContext) -> Result<bool, String> {
        let Some(actual) = context.lookup(self.source, &self.field) else {
            return Err(format!(
                "missing_condition_{}: {}",
                self.source.as_str(),
                self.field
            ));
        };
        self.operator.compare(actual, &self.value).ok_or_else(|| {
            format!(
                "invalid_condition_type: {} expected {} got {}",
                self.field,
                self.value.value_type().as_str(),
                actual.value_type().as_str()
            )
        })
    }
}

/// Source map for a condition field lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyValueSource {
    Input,
    Evidence,
}

impl PolicyValueSource {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Evidence => "evidence",
        }
    }
}

/// Comparison operator for a policy condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOperator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl PolicyOperator {
    fn compare(self, actual: &PolicyValue, expected: &PolicyValue) -> Option<bool> {
        match (actual, expected) {
            (PolicyValue::Boolean(left), PolicyValue::Boolean(right)) => match self {
                Self::Eq => Some(left == right),
                Self::NotEq => Some(left != right),
                Self::Gt | Self::Gte | Self::Lt | Self::Lte => None,
            },
            (PolicyValue::Integer(left), PolicyValue::Integer(right)) => match self {
                Self::Eq => Some(left == right),
                Self::NotEq => Some(left != right),
                Self::Gt => Some(left > right),
                Self::Gte => Some(left >= right),
                Self::Lt => Some(left < right),
                Self::Lte => Some(left <= right),
            },
            (PolicyValue::Text(left), PolicyValue::Text(right)) => match self {
                Self::Eq => Some(left == right),
                Self::NotEq => Some(left != right),
                Self::Gt | Self::Gte | Self::Lt | Self::Lte => None,
            },
            _ => None,
        }
    }

    const fn supports_type(self, value_type: PolicyValueType) -> bool {
        match value_type {
            PolicyValueType::Integer => true,
            PolicyValueType::Boolean | PolicyValueType::Text => {
                matches!(self, Self::Eq | Self::NotEq)
            }
        }
    }
}

/// Data type accepted by a declared policy field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyValueType {
    Boolean,
    Integer,
    Text,
}

impl PolicyValueType {
    fn accepts(self, value: &PolicyValue) -> bool {
        self == value.value_type()
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Text => "text",
        }
    }
}

/// Deterministic scalar values accepted by policy documents and evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum PolicyValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

impl PolicyValue {
    /// Return the scalar type of this value.
    #[must_use]
    pub const fn value_type(&self) -> PolicyValueType {
        match self {
            Self::Boolean(_) => PolicyValueType::Boolean,
            Self::Integer(_) => PolicyValueType::Integer,
            Self::Text(_) => PolicyValueType::Text,
        }
    }
}

/// Caller-provided deterministic evaluation context.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyContext {
    #[serde(default)]
    pub inputs: BTreeMap<String, PolicyValue>,
    #[serde(default)]
    pub evidence: BTreeMap<String, PolicyValue>,
}

impl PolicyContext {
    fn lookup(&self, source: PolicyValueSource, field: &str) -> Option<&PolicyValue> {
        match source {
            PolicyValueSource::Input => self.inputs.get(field),
            PolicyValueSource::Evidence => self.evidence.get(field),
        }
    }
}

/// Evaluation result for a policy and context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyDecision {
    pub policy_id: String,
    pub compatibility_version: CompatibilityVersion,
    pub outputs: BTreeMap<String, PolicyValue>,
    pub fallback_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_rule: Option<String>,
    pub reason: String,
}

/// One validation failure found inside a policy document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyValidationError {
    pub field: String,
    pub message: String,
}

impl PolicyValidationError {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

fn summarize_validation_errors(errors: &[PolicyValidationError]) -> String {
    errors
        .iter()
        .map(|error| format!("{} {}", error.field, error.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn validate_inputs(
    inputs: &[PolicyInput],
    errors: &mut Vec<PolicyValidationError>,
) -> BTreeMap<String, PolicyValueType> {
    let mut seen = BTreeSet::new();
    let mut fields = BTreeMap::new();
    for input in inputs {
        validate_declared_name("inputs", &input.name, &mut seen, errors);
        fields.insert(input.name.clone(), input.value_type);
    }
    fields
}

fn validate_outputs(
    outputs: &[PolicyOutput],
    errors: &mut Vec<PolicyValidationError>,
) -> BTreeMap<String, PolicyValueType> {
    let mut seen = BTreeSet::new();
    let mut fields = BTreeMap::new();
    for output in outputs {
        validate_declared_name("outputs", &output.name, &mut seen, errors);
        fields.insert(output.name.clone(), output.value_type);
    }
    fields
}

fn validate_evidence_fields(
    evidence_fields: &[EvidenceField],
    errors: &mut Vec<PolicyValidationError>,
) -> BTreeMap<String, PolicyValueType> {
    let mut seen = BTreeSet::new();
    let mut fields = BTreeMap::new();
    for evidence in evidence_fields {
        validate_declared_name("evidence_fields", &evidence.name, &mut seen, errors);
        fields.insert(evidence.name.clone(), evidence.value_type);
    }
    fields
}

fn validate_budgets(budgets: &[PolicyBudget], errors: &mut Vec<PolicyValidationError>) {
    let mut seen = BTreeSet::new();
    for budget in budgets {
        validate_declared_name("budgets", &budget.name, &mut seen, errors);
    }
}

fn validate_declared_name(
    scope: &str,
    name: &str,
    seen: &mut BTreeSet<String>,
    errors: &mut Vec<PolicyValidationError>,
) {
    validate_name(scope, name, errors);
    if !seen.insert(name.to_string()) {
        errors.push(PolicyValidationError::new(
            scope,
            format!("duplicate field {name}"),
        ));
    }
}

fn validate_name(scope: &str, name: &str, errors: &mut Vec<PolicyValidationError>) {
    if name.trim().is_empty() {
        errors.push(PolicyValidationError::new(scope, "cannot be empty"));
    }
    if name.contains('\0') {
        errors.push(PolicyValidationError::new(scope, "cannot contain NUL"));
    }
}

fn validate_output_values(
    scope: &str,
    values: &BTreeMap<String, PolicyValue>,
    outputs: &BTreeMap<String, PolicyValueType>,
    require_all: bool,
    errors: &mut Vec<PolicyValidationError>,
) {
    for (name, value) in values {
        let Some(expected_type) = outputs.get(name) else {
            errors.push(PolicyValidationError::new(
                scope,
                format!("unknown output {name}"),
            ));
            continue;
        };
        if !expected_type.accepts(value) {
            errors.push(PolicyValidationError::new(
                scope,
                format!(
                    "{name} expected {} got {}",
                    expected_type.as_str(),
                    value.value_type().as_str()
                ),
            ));
        }
    }

    if require_all {
        for name in outputs.keys() {
            if !values.contains_key(name) {
                errors.push(PolicyValidationError::new(
                    scope,
                    format!("missing fallback output {name}"),
                ));
            }
        }
    }
}

fn validate_rule_conditions(
    rule: &PolicyRule,
    index: usize,
    inputs: &BTreeMap<String, PolicyValueType>,
    evidence_fields: &BTreeMap<String, PolicyValueType>,
    errors: &mut Vec<PolicyValidationError>,
) {
    for (condition_index, condition) in rule.when_all.iter().enumerate() {
        let scope = format!("rules[{index}].when_all[{condition_index}]");
        validate_name(&format!("{scope}.field"), &condition.field, errors);
        let declared_type = match condition.source {
            PolicyValueSource::Input => inputs.get(&condition.field),
            PolicyValueSource::Evidence => evidence_fields.get(&condition.field),
        };
        let Some(declared_type) = declared_type else {
            errors.push(PolicyValidationError::new(
                &scope,
                format!(
                    "unknown {} field {}",
                    condition.source.as_str(),
                    condition.field
                ),
            ));
            continue;
        };
        if !declared_type.accepts(&condition.value) {
            errors.push(PolicyValidationError::new(
                &scope,
                format!(
                    "{} expected {} got {}",
                    condition.field,
                    declared_type.as_str(),
                    condition.value.value_type().as_str()
                ),
            ));
        }
        if !condition.operator.supports_type(*declared_type) {
            errors.push(PolicyValidationError::new(
                &scope,
                format!(
                    "operator {:?} is not supported for {}",
                    condition.operator,
                    declared_type.as_str()
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn field(name: &str, value_type: PolicyValueType, required: bool) -> PolicyInput {
        PolicyInput {
            name: name.to_string(),
            value_type,
            required,
            description: None,
        }
    }

    fn evidence(name: &str, value_type: PolicyValueType, required: bool) -> EvidenceField {
        EvidenceField {
            name: name.to_string(),
            value_type,
            required,
            description: None,
        }
    }

    fn output(name: &str, value_type: PolicyValueType) -> PolicyOutput {
        PolicyOutput {
            name: name.to_string(),
            value_type,
            description: None,
        }
    }

    fn value_text(value: &str) -> PolicyValue {
        PolicyValue::Text(value.to_string())
    }

    fn sample_policy() -> AdaptivePolicy {
        let fallback = BTreeMap::from([
            ("cache_enabled".to_string(), PolicyValue::Boolean(false)),
            (
                "lock_combining_enabled".to_string(),
                PolicyValue::Boolean(false),
            ),
            ("scheduler_weight".to_string(), PolicyValue::Integer(100)),
            ("snapshot_fallback".to_string(), value_text("disabled")),
        ]);

        AdaptivePolicy {
            schema_version: POLICY_SCHEMA_VERSION,
            compatibility_version: CompatibilityVersion {
                family: "swarm_scale".to_string(),
                major: 1,
                minor: 0,
            },
            policy_id: "swarm-scale-defaults".to_string(),
            inputs: vec![
                field("operation", PolicyValueType::Text, true),
                field("core_count", PolicyValueType::Integer, true),
            ],
            outputs: vec![
                output("cache_enabled", PolicyValueType::Boolean),
                output("lock_combining_enabled", PolicyValueType::Boolean),
                output("scheduler_weight", PolicyValueType::Integer),
                output("snapshot_fallback", PolicyValueType::Text),
            ],
            budgets: vec![
                PolicyBudget {
                    name: "evaluation_steps".to_string(),
                    value: 128,
                    unit: BudgetUnit::Count,
                    description: None,
                },
                PolicyBudget {
                    name: "decision_latency".to_string(),
                    value: 1,
                    unit: BudgetUnit::Milliseconds,
                    description: None,
                },
            ],
            evidence_fields: vec![
                evidence("hot_relation_rows", PolicyValueType::Integer, true),
                evidence("p95_lock_wait_ms", PolicyValueType::Integer, true),
                evidence("corpus_hash", PolicyValueType::Text, true),
            ],
            fallback: PolicyFallback {
                outputs: fallback,
                reason: "conservative defaults".to_string(),
            },
            rules: vec![PolicyRule {
                rule_id: "high_contention".to_string(),
                when_all: vec![
                    PolicyCondition {
                        source: PolicyValueSource::Evidence,
                        field: "p95_lock_wait_ms".to_string(),
                        operator: PolicyOperator::Gte,
                        value: PolicyValue::Integer(50),
                    },
                    PolicyCondition {
                        source: PolicyValueSource::Input,
                        field: "core_count".to_string(),
                        operator: PolicyOperator::Gte,
                        value: PolicyValue::Integer(64),
                    },
                ],
                outputs: BTreeMap::from([
                    (
                        "lock_combining_enabled".to_string(),
                        PolicyValue::Boolean(true),
                    ),
                    ("scheduler_weight".to_string(), PolicyValue::Integer(250)),
                ]),
                reason: "contention exceeds deterministic threshold".to_string(),
            }],
        }
    }

    fn valid_context() -> PolicyContext {
        PolicyContext {
            inputs: BTreeMap::from([
                ("operation".to_string(), value_text("ready")),
                ("core_count".to_string(), PolicyValue::Integer(128)),
            ]),
            evidence: BTreeMap::from([
                (
                    "hot_relation_rows".to_string(),
                    PolicyValue::Integer(20_000),
                ),
                ("p95_lock_wait_ms".to_string(), PolicyValue::Integer(72)),
                ("corpus_hash".to_string(), value_text("sha256:abc")),
            ]),
        }
    }

    #[test]
    fn schema_declares_required_policy_sections() {
        let schema = schemars::schema_for!(AdaptivePolicy);
        let value = serde_json::to_value(schema).expect("schema serializes");
        let required = value
            .get("required")
            .and_then(serde_json::Value::as_array)
            .expect("schema has required fields");

        for field in [
            "schema_version",
            "compatibility_version",
            "inputs",
            "outputs",
            "budgets",
            "evidence_fields",
            "fallback",
        ] {
            assert!(
                required.contains(&json!(field)),
                "AdaptivePolicy schema should require {field}"
            );
        }
    }

    #[test]
    fn unknown_policy_fields_are_rejected() {
        let mut value = serde_json::to_value(sample_policy()).expect("policy serializes");
        value["unexpected"] = json!(true);

        let error = serde_json::from_value::<AdaptivePolicy>(value)
            .expect_err("unknown field should fail strict deserialization");
        assert!(
            error.to_string().contains("unknown field"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validation_checks_references_and_fallback_coverage() {
        let mut policy = sample_policy();
        policy
            .fallback
            .outputs
            .insert("unknown_output".to_string(), PolicyValue::Boolean(true));
        policy.rules[0].when_all.push(PolicyCondition {
            source: PolicyValueSource::Evidence,
            field: "missing_metric".to_string(),
            operator: PolicyOperator::Eq,
            value: PolicyValue::Integer(1),
        });

        let errors = policy
            .validate()
            .expect_err("invalid policy should report validation errors");
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("unknown output")),
            "expected unknown output error, got {errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("unknown evidence field")),
            "expected unknown evidence error, got {errors:?}"
        );
    }

    #[test]
    fn missing_required_evidence_fails_closed_to_fallback() {
        let policy = sample_policy();
        let mut context = valid_context();
        context.evidence.remove("p95_lock_wait_ms");

        let decision = policy.evaluate(&context);

        assert!(decision.fallback_active);
        assert_eq!(decision.applied_rule, None);
        assert!(decision.reason.contains("missing_required_evidence"));
        assert_eq!(
            decision.outputs.get("lock_combining_enabled"),
            Some(&PolicyValue::Boolean(false))
        );
    }

    #[test]
    fn invalid_evidence_type_fails_closed_to_fallback() {
        let policy = sample_policy();
        let mut context = valid_context();
        context
            .evidence
            .insert("p95_lock_wait_ms".to_string(), value_text("slow"));

        let decision = policy.evaluate(&context);

        assert!(decision.fallback_active);
        assert!(decision.reason.contains("invalid_evidence"));
        assert_eq!(
            decision.outputs.get("scheduler_weight"),
            Some(&PolicyValue::Integer(100))
        );
    }

    #[test]
    fn matching_rule_overlays_fallback_outputs() {
        let policy = sample_policy();
        let decision = policy.evaluate(&valid_context());

        assert!(!decision.fallback_active);
        assert_eq!(decision.applied_rule.as_deref(), Some("high_contention"));
        assert_eq!(
            decision.outputs.get("lock_combining_enabled"),
            Some(&PolicyValue::Boolean(true))
        );
        assert_eq!(
            decision.outputs.get("cache_enabled"),
            Some(&PolicyValue::Boolean(false))
        );
        assert_eq!(
            decision.outputs.get("scheduler_weight"),
            Some(&PolicyValue::Integer(250))
        );
    }

    #[test]
    fn same_evidence_replays_to_same_decision() {
        let policy = sample_policy();
        let context_a = valid_context();
        let context_b: PolicyContext = serde_json::from_value(json!({
            "evidence": {
                "corpus_hash": "sha256:abc",
                "p95_lock_wait_ms": 72,
                "hot_relation_rows": 20_000
            },
            "inputs": {
                "core_count": 128,
                "operation": "ready"
            }
        }))
        .expect("context parses");

        let decision_a = policy.evaluate(&context_a);
        let decision_b = policy.evaluate(&context_b);

        assert_eq!(decision_a, decision_b);
        assert_eq!(
            serde_json::to_string(&decision_a).expect("decision serializes"),
            serde_json::to_string(&decision_b).expect("decision serializes")
        );
    }
}
