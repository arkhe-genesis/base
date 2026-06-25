//! Tipos para findings.json (alinhado com report-schema.json da Cloudflare)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub verdict: String,
    pub title: String,
    pub severity: Severity,
    pub attack_class: AttackClass,
    pub description: String,
    pub root_cause: Option<String>,
    pub trace: Vec<TraceStep>,
    pub execution: Option<Execution>,
    pub impact: String,
    pub conditions: Option<Conditions>,
    pub remediation: Remediation,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackClass {
    Injection,
    AccessControl,
    ResourceManipulation,
    CryptoAndSecrets,
    BusinessLogic,
    DataExfiltration,
    ChainedAttack,
    Wildcard,
    ObviousStuff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub file: String,
    pub line: usize,
    pub scope: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub method: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditions {
    pub prerequisites: Vec<String>,
    pub attack_vector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remediation {
    pub description: String,
    pub code_changes: Option<String>,
}
