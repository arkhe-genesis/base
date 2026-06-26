use cathedral_tee::{AttestationReport, TEEBridge, TeeType};
use chrono::Utc;

#[test]
fn test_attestation_sgx() {
    let mut bridge = TEEBridge::new();
    bridge.register_trusted_hash("worker1", "abc123");

    let report = AttestationReport {
        worker_id: "worker1".to_string(),
        tee_type: TeeType::SGX,
        binary_hash: "abc123".to_string(),
        config_hash: "".to_string(),
        tee_quote: vec![],
        timestamp: Utc::now().timestamp(),
        nonce: "nonce".to_string(),
        signature: vec![],
        arkhe_version: "1.0".to_string(),
        enclave_measurement: Some("measure".to_string()),
    };

    let result = bridge.verify(&report);
    assert!(result.valid);
}

#[test]
fn test_attestation_fail() {
    let bridge = TEEBridge::new();
    let report = AttestationReport {
        worker_id: "worker2".to_string(),
        tee_type: TeeType::None,
        binary_hash: "bad".to_string(),
        config_hash: "".to_string(),
        tee_quote: vec![],
        timestamp: Utc::now().timestamp(),
        nonce: "nonce".to_string(),
        signature: vec![],
        arkhe_version: "1.0".to_string(),
        enclave_measurement: None,
    };
    let result = bridge.verify(&report);
    assert!(!result.valid);
}
