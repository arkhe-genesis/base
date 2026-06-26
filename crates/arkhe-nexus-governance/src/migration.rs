pub struct MigrationGuide;

pub const STUBS_TO_REPLACE: &[&str] = &[
    "SafeCoreGuard::execute",
    "SafeCoreGuard::update_kernel",
    "SafeCoreGuard::modify_capsule",
    "SafeCoreGuard::update_compliance",
    "NEXUS::admin_action",
    "NEXUS::privileged_operation",
];

pub fn check_migration_status(code: &str) -> Vec<&'static str> {
    STUBS_TO_REPLACE
        .iter()
        .filter(|stub| code.contains(*stub))
        .copied()
        .collect()
}
