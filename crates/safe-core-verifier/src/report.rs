use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub path: String,
    pub language: String,
    pub alpha_hat: f64,
    pub issues: Vec<crate::checks::Issue>,
    pub suggestions: Vec<String>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalReport {
    pub total_files: usize,
    pub passed_files: usize,
    pub failed_files: usize,
    pub files: Vec<FileReport>,
}

impl GlobalReport {
    pub fn from_file_reports(files: Vec<FileReport>) -> Self {
        let total = files.len();
        let passed = files.iter().filter(|f| f.passed).count();
        Self { total_files: total, passed_files: passed, failed_files: total - passed, files }
    }
}
