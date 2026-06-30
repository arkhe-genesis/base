use crate::languages::Language;
use std::path::PathBuf;
use tree_sitter::Tree;

pub struct FileContext {
    pub path: PathBuf,
    pub language: Language,
    pub code: String,
    pub tree: Tree,
    pub content_hash: u64,
}
