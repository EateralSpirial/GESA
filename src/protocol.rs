//! Deterministic release gate. Evidence is attested by the release signer;
//! validating its structure does not execute tests or prove their provenance.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestEvidence {
    pub name: String,
    pub passed: bool,
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub decision: String,
    pub actor: String,
    pub policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Migration {
    pub kind: String,
    pub reversible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retirement {
    pub through_epoch: u32,
    pub migration_complete: bool,
    pub rollback_verified: bool,
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Iteration {
    pub candidate_id: String,
    pub source_revision: String,
    pub artifact_sha256: String,
    pub change_summary: String,
    pub risk: String,
    pub tests: Vec<TestEvidence>,
    pub approval: Approval,
    pub rollback: String,
    pub migration: Migration,
    pub protected_core_changed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retirement: Option<Retirement>,
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|c| c.is_ascii_hexdigit())
}

pub fn validate(value: &Value, sequence: u64, epoch: u32, supports: &[u32]) -> Result<()> {
    let item: Iteration =
        serde_json::from_value(value.clone()).context("invalid iteration evidence schema")?;
    ensure!(sequence > 0, "release sequence must be positive");
    ensure!(epoch > 0, "contract epoch must be positive");
    ensure!(
        nonempty(&item.candidate_id) && nonempty(&item.change_summary),
        "candidate and change summary required"
    );
    ensure!(
        is_hex(&item.source_revision, 40),
        "source revision must be a full Git SHA-1"
    );
    ensure!(
        is_hex(&item.artifact_sha256, 64),
        "tested artifact digest required"
    );
    ensure!(
        matches!(item.risk.as_str(), "low" | "medium" | "high"),
        "invalid risk level"
    );
    ensure!(
        !item.tests.is_empty() && item.tests.len() <= 128,
        "test evidence required (1..128 results)"
    );
    let mut names = std::collections::HashSet::new();
    for test in &item.tests {
        ensure!(
            nonempty(&test.name) && names.insert(&test.name),
            "test names must be present and unique"
        );
        ensure!(
            test.passed && is_hex(&test.evidence_sha256, 64),
            "every required test must pass and have an evidence digest"
        );
    }
    ensure!(
        item.approval.decision == "approved" && nonempty(&item.approval.actor),
        "release approval required"
    );
    match item.approval.policy.as_str() {
        "manual" => (),
        "automatic-low-risk" => ensure!(
            item.risk == "low" && !item.protected_core_changed && item.migration.kind == "none",
            "automatic policy limited to low risk without core changes or migrations"
        ),
        _ => anyhow::bail!("unknown approval policy"),
    }
    ensure!(nonempty(&item.rollback), "rollback plan required");
    ensure!(
        matches!(item.migration.kind.as_str(), "none" | "expand") && item.migration.reversible,
        "prototype requires a reversible, additive migration contract"
    );
    ensure!(
        supports.len() <= 32 && supports.iter().all(|e| *e > 0 && *e <= epoch),
        "invalid supported contract epochs"
    );
    let unique: std::collections::HashSet<_> = supports.iter().collect();
    ensure!(unique.len() == supports.len(), "duplicate contract epochs");
    for required in epoch.saturating_sub(2).max(1)..=epoch {
        ensure!(
            supports.contains(&required),
            "release must preserve contracts from the current and two previous epochs"
        );
    }
    let older = epoch.saturating_sub(3);
    let retained_older = supports.iter().filter(|e| **e <= older).count() as u32;
    if retained_older < older {
        let retirement = item
            .retirement
            .context("contract retirement requires migration and rollback evidence")?;
        ensure!(
            retirement.through_epoch == older
                && retirement.migration_complete
                && retirement.rollback_verified
                && is_hex(&retirement.evidence_sha256, 64),
            "contract retirement gate failed"
        );
        ensure!(
            item.approval.policy == "manual",
            "contract retirement requires manual approval in this prototype"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn good() -> Value {
        serde_json::json!({"candidate_id":"issue-1", "source_revision":"a".repeat(40), "artifact_sha256":"b".repeat(64), "change_summary":"fix retry", "risk":"low", "tests":[{"name":"smoke", "passed":true, "evidence_sha256":"c".repeat(64)}], "approval":{"decision":"approved", "actor":"policy-v1", "policy":"automatic-low-risk"}, "rollback":"keep previous executable", "migration":{"kind":"none", "reversible":true}, "protected_core_changed":false})
    }
    #[test]
    fn accepts_attested_low_risk_release() {
        validate(&good(), 1, 1, &[1]).unwrap();
    }
    #[test]
    fn rejects_failed_test() {
        let mut v = good();
        v["tests"][0]["passed"] = false.into();
        assert!(validate(&v, 1, 1, &[1]).is_err());
    }
    #[test]
    fn rejects_automatic_protected_core_change() {
        let mut v = good();
        v["protected_core_changed"] = true.into();
        assert!(validate(&v, 1, 1, &[1]).is_err());
    }
    #[test]
    fn rejects_automatic_medium_risk() {
        let mut v = good();
        v["risk"] = "medium".into();
        assert!(validate(&v, 1, 1, &[1]).is_err());
    }
    #[test]
    fn manual_core_change_allowed() {
        let mut v = good();
        v["protected_core_changed"] = true.into();
        v["approval"]["policy"] = "manual".into();
        validate(&v, 1, 1, &[1]).unwrap();
    }
    #[test]
    fn preserves_n_plus_two_contract() {
        assert!(validate(&good(), 3, 3, &[2, 3]).is_err());
        validate(&good(), 3, 3, &[1, 2, 3]).unwrap();
        assert!(validate(&good(), 4, 4, &[2, 3, 4]).is_err());
        validate(&good(), 4, 4, &[1, 2, 3, 4]).unwrap();
    }
    #[test]
    fn retirement_requires_evidence_and_manual_gate() {
        let mut v = good();
        v["retirement"] = serde_json::json!({"through_epoch":1,"migration_complete":true,"rollback_verified":true,"evidence_sha256":"d".repeat(64)});
        assert!(validate(&v, 4, 4, &[2, 3, 4]).is_err());
        v["approval"]["policy"] = "manual".into();
        validate(&v, 4, 4, &[2, 3, 4]).unwrap();
    }
    #[test]
    fn rejects_destructive_migration() {
        let mut v = good();
        v["migration"]["kind"] = "contract".into();
        assert!(validate(&v, 1, 1, &[1]).is_err());
    }
    #[test]
    fn rejects_empty_or_duplicate_evidence() {
        let mut v = good();
        v["tests"] = serde_json::json!([]);
        assert!(validate(&v, 1, 1, &[1]).is_err());
    }
    #[test]
    fn rejects_unknown_schema_fields() {
        let mut v = good();
        v["skip_tests"] = true.into();
        assert!(validate(&v, 1, 1, &[1]).is_err());
    }
}
